use std::sync::{Arc, Mutex};

use async_channel::{Receiver, Sender, unbounded};
use gloo_timers::future::TimeoutFuture;
use leptos::{prelude::*, reactive::spawn_local};
use leptos_use::ColorMode;
use serde::{Deserialize, Serialize};
use tracing::info;
use wasm_bindgen::prelude::*;
use web_sys::js_sys::Function;

#[wasm_bindgen(raw_module = "./codemirror.js")]
extern "C" {
    type CM6Editor;

    #[wasm_bindgen(constructor)]
    fn new(id: &str) -> CM6Editor;

    #[wasm_bindgen(method, js_name = "setOnchange")]
    fn set_onchange(this: &CM6Editor, onchange: Function);

    #[wasm_bindgen(method, js_name = "setExec")]
    fn set_exec(this: &CM6Editor, exec: Function);

    #[wasm_bindgen(method, js_name = "setDark")]
    fn set_dark(this: &CM6Editor, dark: bool);

    #[wasm_bindgen(method, js_name = "setReadOnly")]
    fn set_readonly(this: &CM6Editor, readonly: bool);

    #[wasm_bindgen(method, js_name = "getText")]
    fn get_text(this: &CM6Editor) -> String;

    #[wasm_bindgen(method, js_name = "setText")]
    fn set_text(this: &CM6Editor, value: String);

    #[wasm_bindgen(method, js_name = "setKeymap")]
    fn set_keymap(this: &CM6Editor, kbh: &str);
}

// TODO(veluca): rethink this.
pub struct EditorText {
    data: String,
    num_pending_changes: Arc<Mutex<usize>>,
    sender: Sender<()>,
    receiver: Receiver<()>,
}

impl EditorText {
    pub fn from_text(text: String) -> EditorText {
        let (sender, receiver) = unbounded();
        EditorText {
            data: text,
            num_pending_changes: Arc::new(Mutex::new(0)),
            sender,
            receiver,
        }
    }
    pub fn from_str(text: &str) -> EditorText {
        EditorText::from_text(text.to_string())
    }
    pub fn text(&self) -> &String {
        &self.data
    }
    pub async fn await_all_changes(&self) -> () {
        let num_pending_changes = self.num_pending_changes.clone();
        let receiver = self.receiver.clone();
        loop {
            if *num_pending_changes.lock().unwrap() == 0 {
                return;
            }
            receiver.recv().await.expect("sender dropped");
        }
    }
}

#[derive(
    PartialEq, Eq, Clone, Copy, Hash, Debug, Serialize, Deserialize, Default, strum::VariantArray,
)]
pub enum KeyboardMode {
    #[default]
    Standard,
    Vim,
    Emacs,
}

#[component]
pub fn Editor(
    // TODO(veluca): why is this a RwSignal???
    contents: RwSignal<EditorText>,
    name: &'static str,
    #[prop(into)] readonly: Signal<bool>,
    #[prop(optional)] ctrl_enter: Option<Box<dyn Fn()>>,
    #[prop(into)] kb_mode: Signal<KeyboardMode>,
    color_mode: Signal<ColorMode>,
) -> impl IntoView {
    let cm6 = RwSignal::new_local(None);

    let onchange = move |_: JsValue| {
        contents.update_untracked(|val| {
            *val.num_pending_changes.lock().unwrap() += 1;
        });
        spawn_local(async move {
            TimeoutFuture::new(100).await;
            let mut do_update = false;
            contents.update_untracked(|val| {
                let mut v = val.num_pending_changes.lock().unwrap();
                if *v != 0 {
                    *v -= 1;
                    do_update = *v == 0;
                }
            });
            if !do_update {
                return;
            }
            cm6.with_untracked(|x: &Option<CM6Editor>| {
                let Some(cm6) = x else {
                    return;
                };
                let data = cm6.get_text();
                contents.update_untracked(|val| {
                    val.data = data;
                    info!("onchange: {name} {}", val.data.len());
                    // save(cache_key, val);
                })
            });
            let sender = contents.with_untracked(|c| c.sender.clone());
            for _ in 0..sender.receiver_count() {
                sender.send(()).await.expect("receiver dropped");
            }
        });
    };

    let id = format!("{name}-editor");
    {
        let id = id.clone();
        queue_microtask(move || {
            let editor = CM6Editor::new(&id);
            if let Some(ctrl_enter) = ctrl_enter {
                editor.set_exec(Closure::wrap(ctrl_enter).into_js_value().unchecked_into());
            }
            editor.set_onchange(
                Closure::<dyn Fn(_)>::new(onchange)
                    .into_js_value()
                    .unchecked_into(),
            );
            cm6.set(Some(editor));
        });
    }

    Effect::new(move |_| {
        cm6.with(|x| {
            let Some(cm6) = x else {
                return;
            };
            cm6.set_dark(color_mode.get() != ColorMode::Light);
        });
    });

    Effect::new(move |_| {
        cm6.with(|x| {
            let Some(cm6) = x else {
                return;
            };
            cm6.set_text(contents.with(|x| x.text().to_string()));
        });
    });

    Effect::new(move |_| {
        cm6.with(|x| {
            let Some(cm6) = x else {
                return;
            };
            cm6.set_readonly(readonly.get());
        });
    });

    Effect::new(move |_| {
        cm6.with(|x| {
            let Some(cm6) = x else {
                return;
            };
            match kb_mode.get() {
                KeyboardMode::Standard => {
                    cm6.set_keymap("");
                }
                KeyboardMode::Vim => cm6.set_keymap("vim"),
                KeyboardMode::Emacs => cm6.set_keymap("emacs"),
            }
        });
    });

    view! { <div id=id style="height: 100%; width: 100%; max-height: 75vh; font-size: 1.2em;"></div> }
}
