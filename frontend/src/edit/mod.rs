use leptos::prelude::*;
use leptos::server::codee::string::JsonSerdeCodec;
use leptos_use::storage::use_local_storage;
use leptos_use::{UseColorModeOptions, use_color_mode_with_options};
use thaw::{Flex, Layout, LayoutHeader};

use crate::compilation_manager::CompilationManager;
use crate::compilation_results::CompilationResults;
use crate::editor::{Editor, KeyboardMode};
use crate::header::Header;

#[component]
pub fn EditPage() -> impl IntoView {
    let color_mode = use_color_mode_with_options(
        UseColorModeOptions::default()
            .cookie_enabled(true)
            .cookie_name("typst-translation-color-mode"),
    );

    let (kb_mode, set_kb_mode, _) =
        use_local_storage::<KeyboardMode, JsonSerdeCodec>("typst-translation-kb-mode");

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
        <Layout attr:style="height: 100vh">
            <LayoutHeader>
                <Header kb_mode=(kb_mode, set_kb_mode) />
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
                />
                <CompilationResults
                    results=compilation_manager.get_result()
                    attr:style="width: 50%; height: calc(100vh - 65px);"
                />
            </Flex>
        </Layout>
    }
}
