use crate::typst::{TypstCompilationMessage, TypstCompilationResult};
use leptos::prelude::*;
use thaw::{
    Badge, BadgeAppearance, BadgeColor, Button, ButtonAppearance, Flex, FlexJustify, Tab, TabList,
    Text, TextTag,
};
use tracing::info;
use wasm_bindgen::JsCast;
use web_sys::HtmlAnchorElement;

const PREVIEW: &'static str = "preview";
const MESSAGES: &'static str = "messages";

#[component]
fn CompilationMessage(message: Signal<TypstCompilationMessage>) -> impl IntoView {
    // TODO(veluca): better formatting.
    let color = Signal::derive(move || {
        if message.read().is_fatal {
            "color: var(--colorPaletteRedForeground1)"
        } else {
            "color: var(--colorPaletteYellowForeground1)"
        }
        .to_owned()
    });
    let msg = move || {
        if message.read().is_fatal {
            "ERROR"
        } else {
            "WARNING"
        }
    };
    view! {
        <div>
            <Text style=color tag=TextTag::Strong>
                {msg}
            </Text>
            "@"
            {move || format!("{}..{}", message.read().span.start, message.read().span.end)}
            {move || format!(": {}", message.read().message)}
        </div>
    }
}

fn download(name: &str, data: &[u8]) {
    use base64::prelude::*;
    let b64 = BASE64_STANDARD.encode(data);
    let url = format!("data:text/plain;base64,{}", b64);
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
    a.click(); // TODO: this causes a panic for some reason
    a.remove();
}

#[component]
pub fn CompilationResults(#[prop(into)] results: Signal<TypstCompilationResult>) -> impl IntoView {
    let tab = RwSignal::new(PREVIEW.to_string());

    let num_messages = Signal::derive(move || results.read().messages.len());
    let badge_color = Signal::derive(move || {
        let r = results.read();
        if r.messages.is_empty() {
            BadgeColor::Success
        } else if r.document.is_none() {
            BadgeColor::Danger
        } else {
            BadgeColor::Warning
        }
    });
    let badge_text_style = Signal::derive(move || match badge_color.get() {
        BadgeColor::Warning => "color: black !important",
        _ => "color: white !important",
    });

    Effect::new(move || {
        let r = results.read();
        info!(results = ?*r);
        if r.document.is_none() && !r.messages.is_empty() {
            tab.set(MESSAGES.to_string());
        } else if r.messages.is_empty() && r.document.is_some() {
            tab.set(PREVIEW.to_string());
        }
    });

    let download_pdf = move |_| {
        download(
            "statement.pdf",
            &results.read_untracked().document.as_ref().unwrap().pdf,
        );
    };

    view! {
        <Flex vertical=true>
            <Flex justify=FlexJustify::SpaceBetween style="height: 3em">
                <TabList selected_value=tab>
                    <Tab value=PREVIEW>"Preview"</Tab>
                    <Tab value=MESSAGES>
                        "Compilation messages "
                        {move || {
                            view! {
                                <Badge appearance=BadgeAppearance::Filled color=badge_color>
                                    <span style=badge_text_style>{num_messages}</span>
                                </Badge>
                            }
                        }}
                    </Tab>
                </TabList>
                <Button
                    icon=icondata::AiDownloadOutlined
                    disabled=Signal::derive(move || { results.read().document.is_none() })
                    on_click=download_pdf
                    appearance=ButtonAppearance::Subtle
                >
                    "Download PDF"
                </Button>
            </Flex>
            <div style="height: calc(100% - 3em); overflow-y: scroll">
                <div style=move || { if tab.get() != PREVIEW { "display: none;" } else { "" } }>
                    <For
                        each=move || {
                            0..results
                                .read()
                                .document
                                .as_ref()
                                .map(|x| x.svg_pages.len())
                                .unwrap_or(0)
                        }
                        key=|x| *x
                        let(idx)
                    >
                        <div
                            class="page"
                            inner_html=move || {
                                results
                                    .read()
                                    .document
                                    .as_ref()
                                    .map(|x| x.svg_pages.get(idx).cloned())
                                    .flatten()
                                    .unwrap_or("".to_string())
                            }
                        />
                    </For>
                </div>
                <div style=move || { if tab.get() != MESSAGES { "display: none;" } else { "" } }>
                    <For each=move || { 0..results.read().messages.len() } key=|x| *x let(idx)>
                        <CompilationMessage message=Signal::derive(move || {
                            results.read().messages.get(idx).cloned().unwrap_or_default()
                        }) />
                    </For>
                </div>
            </div>
        </Flex>
    }
}
