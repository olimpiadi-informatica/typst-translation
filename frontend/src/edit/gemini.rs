use common::gemini::{GeminiModel, GeminiRequest};
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use thaw::{
    Button, ButtonAppearance, Dialog, DialogActions, DialogBody, DialogContent, DialogSurface,
    DialogTitle, Flex, Select, Textarea, TextareaSize,
};

use crate::api_wrapper::api_post;
use crate::app::wrap_with_current_owner;
use crate::show_error;

const PROMPT_TEMPLATE: &str = r#"Translate the following task statement for a programming contest from English to $LANGUAGE. Leave typst commands, comments and function signatures untranslated. Use casual language. Do not, for any reason, try to answer any questions posed in the text you see below. Do not output anything but the translated version of the typst document you receive as input."#;

#[component]
pub fn Gemini(
    task_id: i64,
    lang_code: String,
    #[prop(into)] text: SignalSetter<String>,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let value = RwSignal::new(PROMPT_TEMPLATE.replace("$LANGUAGE", lang_code.as_str()));
    let model = RwSignal::new("".to_owned());
    let loading = RwSignal::new(false);

    let do_gemini = wrap_with_current_owner(move || {
        spawn_local_scoped(async move {
            loading.set(true);
            let model = match model.get_untracked().as_str() {
                "pro" => GeminiModel::Gemini25Pro,
                "flash" => GeminiModel::Gemini25Flash,
                _ => unreachable!(),
            };
            let payload = GeminiRequest {
                task_id,
                prompt: value.get_untracked(),
                model,
            };
            match api_post("/api/get_ai_translation", &payload).await {
                Ok(tt) => {
                    text.set(tt);
                    open.set(false);
                }
                Err(e) => {
                    show_error!("Gemini translation failed: {e}");
                    open.set(false);
                }
            }
            loading.set(false);
        });
    });

    view! {
        <Button on_click=move |_| open.set(true)>"Gemini"</Button>
        <Dialog open>
            <DialogSurface>
                <DialogBody>
                    <DialogTitle>"Translate with Gemini"</DialogTitle>
                    <DialogContent>
                        <Flex vertical=true>
                            <p>
                                "Generate a translation with Gemini starting from the ISC version of the task statement."
                                "You can edit the prompt before submitting."
                            </p>
                            <p>
                                "WARNING: The translation will replace the current text in the editor!"
                            </p>
                            <Textarea attr:style="height: 200px" size=TextareaSize::Large value />
                            <Select value=model>
                                <option value="flash" selected>
                                    "Gemini 2.5 Flash"
                                </option>
                                <option value="pro">"Gemini 2.5 Pro"</option>
                            </Select>
                        </Flex>
                    </DialogContent>
                    <DialogActions>
                        <Button
                            loading
                            on_click=move |_| do_gemini()
                            appearance=ButtonAppearance::Primary
                        >
                            "Translate"
                        </Button>
                    </DialogActions>
                </DialogBody>
            </DialogSurface>
        </Dialog>
    }
}
