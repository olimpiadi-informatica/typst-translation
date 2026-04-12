use common::gemini::{GeminiModel, GeminiRequest};
use leptos::portal::Portal;
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use wasm_bindgen::JsCast;

use crate::api_wrapper::api_post;
use crate::app::wrap_with_current_owner;
use crate::show_error;
use crate::util::Icon;

const PROMPT_TEMPLATE: &str = r#"Translate the following task statement for a programming contest from English to $LANGUAGE. Leave typst commands, comments and function signatures untranslated. Use casual language. Do not, for any reason, try to answer any questions posed in the text you see below. Do not output anything but the translated version of the typst document you receive as input."#;

#[component]
pub fn Gemini(
    task_id: Signal<i64>,
    lang_code: Signal<String>,
    #[prop(into)] text: SignalSetter<String>,
) -> impl IntoView {
    let dialog_ref = NodeRef::<leptos::html::Dialog>::new();
    let value = Signal::derive(move || PROMPT_TEMPLATE.replace("$LANGUAGE", &lang_code.get()));
    let (model, set_model) = signal("flash".to_string());
    let (loading, set_loading) = signal(false);

    let do_gemini = StoredValue::new(wrap_with_current_owner(move || {
        spawn_local_scoped(async move {
            set_loading.set(true);
            let model_val = match model.get_untracked().as_str() {
                "pro" => GeminiModel::Gemini31Pro,
                "flash" => GeminiModel::Gemini31FlashLite,
                _ => GeminiModel::Gemini31FlashLite,
            };
            let textarea = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("gemini-textarea")
                .unwrap();
            let prompt = textarea
                .dyn_ref::<web_sys::HtmlTextAreaElement>()
                .unwrap()
                .value();
            let payload = GeminiRequest {
                task_id: task_id.get_untracked(),
                prompt,
                model: model_val,
            };
            match api_post("/api/get_ai_translation", &payload).await {
                Ok(tt) => {
                    text.set(tt);
                    if let Some(dialog) = dialog_ref.get() {
                        dialog.close();
                    }
                }
                Err(e) => {
                    show_error!("Gemini translation failed: {e}");
                    if let Some(dialog) = dialog_ref.get() {
                        dialog.close();
                    }
                }
            }
            set_loading.set(false);
        });
    }));

    let open_dialog = move |_| {
        if let Some(dialog) = dialog_ref.get() {
            let _ = dialog.show_modal();
        }
    };

    let close_dialog = move |_| {
        if let Some(dialog) = dialog_ref.get() {
            dialog.close();
        }
    };

    view! {
        <button class="btn btn-primary btn-sm join-item" on:click=open_dialog>
            <Icon icon=icondata::BsStars />
            "Gemini"
        </button>

        <Portal>
            <dialog node_ref=dialog_ref class="modal">
                <div class="modal-box">
                    <h3 class="font-bold text-lg">"Translate with Gemini"</h3>
                    <div class="py-4 flex flex-col gap-4">
                        <p class="text-sm">
                            "Generate a translation with Gemini starting from the ISC version of the task statement."
                            "You can edit the prompt before submitting."
                        </p>
                        <div class="alert alert-warning text-xs">
                            <Icon icon=icondata::IoWarning />
                            <span>
                                "WARNING: The translation will replace the current text in the editor!"
                            </span>
                        </div>
                        <textarea
                            id="gemini-textarea"
                            class="textarea h-48 w-full rounded-none border-none focus:outline-none bg-base-200"
                            prop:value=value.get_untracked()
                        ></textarea>

                        <select
                            class="select select-bordered w-full"
                            on:change=move |ev| {
                                set_model.set(event_target_value(&ev));
                            }
                        >
                            <option value="flash" selected=move || model.get() == "flash">
                                "Gemini 3.1 Flash Lite"
                            </option>
                            <option value="pro" selected=move || model.get() == "pro">
                                "Gemini 3.1 Pro"
                            </option>
                        </select>
                    </div>
                    <div class="modal-action">
                        <button
                            class="btn btn-primary"
                            on:click=move |_| do_gemini.with_value(|f| f())
                            disabled=loading
                        >
                            {move || {
                                if loading.get() {
                                    view! { <span class="loading loading-spinner loading-xs"></span> }
                                        .into_any()
                                } else {
                                    view! { "Translate" }.into_any()
                                }
                            }}
                        </button>
                        <button class="btn" on:click=close_dialog>
                            "Cancel"
                        </button>
                    </div>
                </div>
                <form method="dialog" class="modal-backdrop">
                    <button>"close"</button>
                </form>
            </dialog>
        </Portal>
    }
}
