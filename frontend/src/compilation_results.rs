use leptos::prelude::*;
use tracing::info;
use wasm_bindgen::JsCast;
use web_sys::HtmlAnchorElement;

use crate::typst::{TypstCompilationMessage, TypstCompilationResult};
use crate::util::Icon;

const PREVIEW: &str = "preview";
const MESSAGES: &str = "messages";

#[component]
fn CompilationMessage(message: Signal<TypstCompilationMessage>) -> impl IntoView {
    let alert_class = move || {
        if message.read().is_fatal {
            "text-error"
        } else {
            "text-warning"
        }
    };
    let msg = move || {
        if message.read().is_fatal {
            "ERROR"
        } else {
            "WARNING"
        }
    };
    view! {
        <div class="mb-2 text-sm">
            <strong class=alert_class>{msg}</strong>
            <span class="opacity-70 mx-1">"@"</span>
            <code class="bg-base-200 px-1 rounded">
                {move || {
                    let m = message.get();
                    format!("{}..{}", m.span.start, m.span.end)
                }}
            </code>
            <span class="ml-2">{move || message.get().message.to_string()}</span>
        </div>
    }
}

fn download(name: &str, data: &[u8]) {
    use base64::prelude::*;
    let b64 = BASE64_STANDARD.encode(data);
    let url = format!("data:application/pdf;base64,{}", b64);
    let w = window();
    let d = w.document().expect("no document");
    let a = d
        .create_element("a")
        .unwrap()
        .dyn_into::<HtmlAnchorElement>()
        .unwrap();
    a.set_download(name);
    a.set_href(&url);
    d.body().expect("no body").append_child(&a).unwrap();
    a.click();
    a.remove();
}

#[component]
pub fn CompilationResults(
    #[prop(into)] results: Signal<TypstCompilationResult>,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let (tab, set_tab) = signal(PREVIEW.to_string());

    let num_messages = Signal::derive(move || results.read().messages.len());
    let badge_class = Signal::derive(move || {
        let r = results.read();
        if r.messages.is_empty() {
            "badge-success"
        } else if r.document.is_none() {
            "badge-error"
        } else {
            "badge-warning"
        }
    });

    Effect::new(move || {
        let r = results.read();
        info!(results = ?*r);
        if r.document.is_none() && !r.messages.is_empty() {
            set_tab.set(MESSAGES.to_string());
        } else if r.messages.is_empty() && r.document.is_some() {
            set_tab.set(PREVIEW.to_string());
        }
    });

    let download_pdf = move |_| {
        if let Some(doc) = &results.read_untracked().document {
            download("statement.pdf", &doc.pdf);
        }
    };

    view! {
        <div class=move || {
            format!(
                "flex flex-col bg-base-200 {}",
                class.as_ref().cloned().unwrap_or_default(),
            )
        }>
            <div class="flex justify-between items-center p-2 bg-base-100 border-b border-base-300">
                <div class="tabs tabs-boxed">
                    <button
                        class="tab"
                        class:tab-active=move || tab.get() == PREVIEW
                        on:click=move |_| set_tab.set(PREVIEW.to_string())
                    >
                        "Preview"
                    </button>
                    <button
                        class="tab gap-2"
                        class:tab-active=move || tab.get() == MESSAGES
                        on:click=move |_| set_tab.set(MESSAGES.to_string())
                    >
                        "Messages"
                        <span class=move || format!("badge badge-sm {}", badge_class.get())>
                            {num_messages}
                        </span>
                    </button>
                </div>
                <button
                    class="btn btn-ghost btn-sm gap-2"
                    disabled=move || results.read().document.is_none()
                    on:click=download_pdf
                >
                    <Icon icon=icondata::AiDownloadOutlined />
                    "PDF"
                </button>
            </div>
            <div class="flex-1 overflow-y-auto overflow-x-hidden p-4">
                <div class:hidden=move || tab.get() != PREVIEW>
                    <For
                        each=move || {
                            let len = results
                                .read()
                                .document
                                .as_ref()
                                .map(|x| x.svg_pages.len())
                                .unwrap_or(0);
                            0..len
                        }
                        key=|x| *x
                        let(idx)
                    >
                        <div
                            class="page-svg-container bg-white mb-4 mx-auto max-w-full"
                            inner_html=move || {
                                results
                                    .read()
                                    .document
                                    .as_ref()
                                    .and_then(|x| x.svg_pages.get(idx).cloned())
                                    .unwrap_or_default()
                            }
                        />
                    </For>
                </div>
                <div
                    class:hidden=move || tab.get() != MESSAGES
                    class="bg-base-100 p-4 rounded-lg shadow-inner"
                >
                    <For
                        each=move || { 0..results.read().messages.len() }
                        key=|x| *x
                        let(idx)
                    >
                        <CompilationMessage message=Signal::derive(move || {
                            results.read().messages.get(idx).cloned().unwrap_or_default()
                        }) />
                    </For>
                    <Show when=move || results.read().messages.is_empty()>
                        <p class="text-success text-center italic">"No compilation messages."</p>
                    </Show>
                </div>
            </div>
        </div>
    }
}
