use std::time::Duration;

use async_channel::{Receiver, Sender, bounded};
use futures_util::StreamExt;
use gloo_timers::future::sleep;
use gloo_worker::Spawnable;
use leptos::{prelude::*, reactive::spawn_local};
use tracing::info;
use web_time::Instant;

use crate::{TypstWorker, typst::TypstCompilationResult};

pub enum CompilationStatus {
    Ready,
    OutOfDate,
    CompilationFailure,
}

const COMPILE_MIN_INTERVAL: Duration = Duration::from_millis(50);
const LIVE_COMPILE_FAILURE_DELAY: Duration = Duration::from_secs(3);

#[derive(Debug, Clone)]
pub struct CompilationManager {
    result: RwSignal<TypstCompilationResult>,
    got_manual_request: RwSignal<bool>,
    sender: Sender<()>,
    status: RwSignal<CompilationStatus>,
    epoch: RwSignal<usize>,
    wait_until: RwSignal<Instant>,
    input_text: RwSignal<String>,
}

impl CompilationManager {
    pub fn new(input_text: RwSignal<String>) -> CompilationManager {
        let (sender, recv) = bounded(1);
        let ret = CompilationManager {
            result: RwSignal::new(TypstCompilationResult::default()),
            got_manual_request: RwSignal::new(false),
            sender,
            status: RwSignal::new(CompilationStatus::OutOfDate),
            epoch: RwSignal::new(0),
            wait_until: RwSignal::new(Instant::now()),
            input_text,
        };
        spawn_local(ret.clone().compile_loop(recv));
        ret
    }

    pub fn get_result(&self) -> Signal<TypstCompilationResult> {
        self.result.read_only().into()
    }

    pub fn get_status(&self) -> Signal<CompilationStatus> {
        self.status.read_only().into()
    }

    pub fn do_compile(&self, manual_request: bool) {
        if manual_request {
            self.got_manual_request.set(true);
        }
        self.status.set(CompilationStatus::OutOfDate);
        self.epoch.update(|x| *x += 1);
        let _ = self.sender.try_send(()); // ignore channel being full.
    }

    async fn compile_loop(self, compilation_requested: Receiver<()>) {
        let mut typst_worker =
            TypstWorker::spawner().spawn_with_loader("typst_translation_worker_loader.js");
        loop {
            compilation_requested.recv().await.unwrap();
            let mut got_manual_request = *self.got_manual_request.read_untracked();
            if !got_manual_request {
                if let Some(delay) = self
                    .wait_until
                    .get_untracked()
                    .checked_duration_since(Instant::now())
                {
                    sleep(delay).await;
                }
                // Flush additional requests that happened while waiting.
                let _ = compilation_requested.try_recv();
                got_manual_request = *self.got_manual_request.read_untracked();
            }
            if got_manual_request {
                self.got_manual_request.set(false);
            }
            self.wait_until.set(Instant::now() + COMPILE_MIN_INTERVAL);
            let epoch = self.epoch.get_untracked();
            let input = self.input_text.get_untracked().as_bytes().to_vec();
            typst_worker.send_input(input);
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
        info!(epoch, cur_epoch = self.epoch.get_untracked());
        if self.epoch.get_untracked() != epoch {
            return;
        }
        self.status.set(if result.document.is_some() {
            CompilationStatus::Ready
        } else {
            CompilationStatus::CompilationFailure
        });
        self.result.set(result);
    }
}
