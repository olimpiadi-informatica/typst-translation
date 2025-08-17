use crate::{
    compilation_results::CompilationResults,
    editor::{Editor, EditorText, KeyboardMode},
    header::Header,
    typst::TypstWorker,
};
use async_channel::unbounded;
use futures_util::StreamExt;
use gloo_worker::Spawnable;
use leptos::{prelude::*, reactive::spawn_local, server::codee::string::JsonSerdeCodec};
use leptos_use::{
    ColorMode, UseColorModeOptions, storage::use_local_storage, use_color_mode_with_options,
};
use thaw::{ConfigProvider, Flex, Layout, LayoutHeader, Theme};

fn theme_from_color_mode(color_mode: ColorMode) -> Theme {
    if color_mode == ColorMode::Dark {
        Theme::dark()
    } else {
        Theme::light()
    }
}

#[component]
pub fn App() -> impl IntoView {
    let color_mode = use_color_mode_with_options(
        UseColorModeOptions::default()
            .cookie_enabled(true)
            .cookie_name("typst-translation-color-mode"),
    );

    let (kb_mode, set_kb_mode, _) =
        use_local_storage::<KeyboardMode, JsonSerdeCodec>("typst-translation-kb-mode");

    let theme = RwSignal::new(Theme::dark());

    Effect::new(move || {
        theme.set(theme_from_color_mode(color_mode.mode.get()));
    });

    let text = RwSignal::new(EditorText::from_str(""));

    let compilation_result = RwSignal::new(Default::default());

    let (send_compile, recv_compile) = unbounded();
    spawn_local(async move {
        let mut typst_worker =
            TypstWorker::spawner().spawn_with_loader("typst_translation_worker_loader.js");
        loop {
            recv_compile.recv().await.unwrap();
            text.read_untracked().await_all_changes().await;
            let input = text.read_untracked().text().as_bytes().to_vec();
            typst_worker.send_input(input);
            let response = typst_worker.next().await.unwrap();
            compilation_result.set(response);
        }
    });

    // Trigger an initial compilation.
    // TODO(veluca): instead auto-refresh?
    send_compile.try_send(()).unwrap();

    view! {
        <ConfigProvider theme>
            <Layout attr:style="height: 100vh">
                <LayoutHeader attr:style="height: 64px; padding: 0 20px; display: flex; align-items: center; justify-content: space-between;">
                    <Header
                        color_mode=(color_mode.mode, color_mode.set_mode)
                        kb_mode=(kb_mode, set_kb_mode)
                    />
                </LayoutHeader>
                <Flex>
                    <Editor
                        contents=text
                        name="statement-editor"
                        readonly=false
                        ctrl_enter=Box::new(move || send_compile.try_send(()).unwrap())
                        kb_mode
                        color_mode=color_mode.mode
                        attr:style="width: 50%; height: calc(100vh - 65px);"
                    ></Editor>
                    <CompilationResults
                        results=compilation_result
                        attr:style="width: 50%; height: calc(100vh - 65px);"
                    ></CompilationResults>
                </Flex>
            </Layout>
        </ConfigProvider>
    }
}
