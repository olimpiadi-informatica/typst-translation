use crate::{
    compilation_manager::CompilationManager,
    compilation_results::CompilationResults,
    editor::{Editor, KeyboardMode},
    header::Header,
};
use leptos::{prelude::*, server::codee::string::JsonSerdeCodec};
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

    let text = RwSignal::new(String::new());

    let compilation_manager = CompilationManager::new(text);

    let on_change = {
        let compilation_manager = compilation_manager.clone();
        Box::new(move || compilation_manager.do_compile(false))
    };

    let ctrl_enter = {
        let compilation_manager = compilation_manager.clone();
        Box::new(move || compilation_manager.do_compile(true))
    };

    // Compile initial state.
    compilation_manager.do_compile(true);

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
                        ctrl_enter
                        on_change
                        kb_mode
                        color_mode=color_mode.mode
                        attr:style="width: 50%; height: calc(100vh - 65px);"
                    ></Editor>
                    <CompilationResults
                        results=compilation_manager.get_result()
                        attr:style="width: 50%; height: calc(100vh - 65px);"
                    ></CompilationResults>
                </Flex>
            </Layout>
        </ConfigProvider>
    }
}
