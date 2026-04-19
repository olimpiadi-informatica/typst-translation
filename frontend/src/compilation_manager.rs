use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_channel::{Receiver, Sender, bounded};
use futures_util::StreamExt;
use gloo_timers::future::sleep;
use gloo_worker::Spawnable;
use leptos::prelude::*;
use leptos::reactive::spawn_local;
use tracing::info;
use web_time::Instant;

use crate::TypstWorker;
use crate::typst::{TypstCompilationResult, TypstWorkerInput};

pub enum CompilationStatus {
    Ready,
    OutOfDate,
    CompilationFailure,
}

const COMPILE_MIN_INTERVAL: Duration = Duration::from_millis(50);
const LIVE_COMPILE_FAILURE_DELAY: Duration = Duration::from_secs(3);

#[derive(Debug)]
pub struct CompilationManagerInner {
    result: RwSignal<TypstCompilationResult>,
    got_manual_request: RwSignal<bool>,
    sender: Sender<()>,
    status: RwSignal<CompilationStatus>,
    epoch: RwSignal<usize>,
    wait_until: RwSignal<Instant>,
    inputs: Mutex<HashMap<PathBuf, Signal<Vec<u8>>>>,
    extra_fonts: Mutex<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct CompilationManager(Arc<CompilationManagerInner>);

impl CompilationManager {
    pub fn new() -> CompilationManager {
        let (sender, recv) = bounded(1);
        let ret = CompilationManager(Arc::new(CompilationManagerInner {
            result: RwSignal::new(TypstCompilationResult::default()),
            got_manual_request: RwSignal::new(false),
            sender,
            status: RwSignal::new(CompilationStatus::OutOfDate),
            epoch: RwSignal::new(0),
            wait_until: RwSignal::new(Instant::now()),
            inputs: Mutex::new(HashMap::new()),
            extra_fonts: Mutex::new(Vec::new()),
        }));
        spawn_local(ret.clone().compile_loop(recv));
        ret
    }

    pub fn set_inputs(&self, inputs: HashMap<PathBuf, Signal<Vec<u8>>>) {
        *self.0.inputs.lock().unwrap() = inputs;
    }

    pub fn set_extra_fonts(&self, extra_fonts: Vec<String>) {
        *self.0.extra_fonts.lock().unwrap() = extra_fonts;
    }

    pub fn get_result(&self) -> Signal<TypstCompilationResult> {
        self.0.result.read_only().into()
    }

    #[allow(dead_code)]
    pub fn get_status(&self) -> Signal<CompilationStatus> {
        self.0.status.read_only().into()
    }

    pub fn do_compile(&self, manual_request: bool) {
        if manual_request {
            self.0.got_manual_request.set(true);
        }
        self.0.status.set(CompilationStatus::OutOfDate);
        self.0.epoch.update(|x| *x += 1);
        let _ = self.0.sender.try_send(()); // ignore channel being full.
    }

    async fn compile_loop(self, compilation_requested: Receiver<()>) {
        let mut typst_worker =
            TypstWorker::spawner().spawn_with_loader("/typst_translation_worker_loader.js");
        loop {
            compilation_requested.recv().await.unwrap();
            let mut got_manual_request = *self.0.got_manual_request.read_untracked();
            if !got_manual_request {
                if let Some(delay) = self
                    .0
                    .wait_until
                    .get_untracked()
                    .checked_duration_since(Instant::now())
                {
                    sleep(delay).await;
                }
                // Flush additional requests that happened while waiting.
                let _ = compilation_requested.try_recv();
                got_manual_request = *self.0.got_manual_request.read_untracked();
            }
            if got_manual_request {
                self.0.got_manual_request.set(false);
            }
            self.0.wait_until.set(Instant::now() + COMPILE_MIN_INTERVAL);
            let epoch = self.0.epoch.get_untracked();
            let files = self
                .0
                .inputs
                .lock()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.get_untracked().to_vec()))
                .collect::<HashMap<_, _>>();
            let extra_fonts = self.0.extra_fonts.lock().unwrap().clone();
            typst_worker.send_input(TypstWorkerInput { files, extra_fonts });
            let response = typst_worker.next().await.unwrap();
            if got_manual_request || response.document.is_some() {
                self.set_result(response, epoch);
            } else {
                let this = self.clone();
                spawn_local(async move {
                    sleep(LIVE_COMPILE_FAILURE_DELAY).await;
                    this.set_result(response, epoch);
                });
            }
        }
    }

    fn set_result(&self, result: TypstCompilationResult, epoch: usize) {
        info!(epoch, cur_epoch = self.0.epoch.get_untracked());
        if self.0.epoch.get_untracked() != epoch {
            return;
        }
        self.0.status.set(if result.document.is_some() {
            CompilationStatus::Ready
        } else {
            CompilationStatus::CompilationFailure
        });
        self.0.result.set(result);
    }
}
