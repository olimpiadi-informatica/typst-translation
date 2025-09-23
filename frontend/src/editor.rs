use leptos::prelude::*;
use leptos_use::ColorMode;
use serde::{Deserialize, Serialize};
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
    contents: RwSignal<String>,
    name: &'static str,
    #[prop(into)] readonly: Signal<bool>,
    #[prop(optional)] ctrl_enter: Option<Box<dyn Fn()>>,
    #[prop(optional)] on_change: Option<Box<dyn Fn()>>,
    #[prop(into)] kb_mode: Signal<KeyboardMode>,
    color_mode: Signal<ColorMode>,
) -> impl IntoView {
    let cm6 = RwSignal::new_local(None);

    let onchange = {
        move |_: JsValue| {
            if let Some(on_change) = on_change.as_ref() {
                on_change();
            }
            cm6.with_untracked(|x: &Option<CM6Editor>| {
                let Some(cm6) = x else {
                    return;
                };
                let data = cm6.get_text();
                contents.update(|val| {
                    *val = data;
                })
            });
        }
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
            cm6.set_text(contents.get_untracked().clone());
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
                KeyboardMode::Standard => cm6.set_keymap(""),
                KeyboardMode::Vim => cm6.set_keymap("vim"),
                KeyboardMode::Emacs => cm6.set_keymap("emacs"),
            }
        });
    });

    view! { <div id=id style="height: 100%; width: 100%; max-height: 75vh; font-size: 1.2em;"></div> }
}
