use leptos::ev;
use leptos::prelude::*;
use leptos::server::codee::string::JsonSerdeCodec;
use leptos_use::storage::use_local_storage;
use leptos_use::{ColorMode, use_event_listener};

use crate::compilation_manager::CompilationManager;
use crate::compilation_results::CompilationResults;
use crate::editor::{Editor, KeyboardMode};

#[component]
pub fn SplitEditorLayout(
    contents: RwSignal<String>,
    readonly: Memo<bool>,
    on_change: Box<dyn Fn()>,
    ctrl_enter: Box<dyn Fn()>,
    kb_mode: Signal<KeyboardMode>,
) -> impl IntoView {
    let (split_width, set_split_width, _) =
        use_local_storage::<f64, JsonSerdeCodec>("typst-translation-split-width");

    if split_width.get_untracked() == 0.0 {
        set_split_width.set(50.0);
    }

    let is_dragging = RwSignal::new(false);
    let _ = use_event_listener(window(), ev::mousemove, move |ev: web_sys::MouseEvent| {
        if is_dragging.get_untracked() {
            let width = web_sys::window()
                .unwrap()
                .inner_width()
                .unwrap()
                .as_f64()
                .unwrap();
            let x = ev.client_x() as f64;
            let percent = (x / width) * 100.0;
            let percent = percent.clamp(33.0, 66.0);
            set_split_width.set(percent);
        }
    });

    let _ = use_event_listener(window(), ev::mouseup, move |_| {
        is_dragging.set(false);
    });

    let compilation_manager = expect_context::<CompilationManager>();
    let color_mode = expect_context::<Signal<ColorMode>>();

    view! {
        <div
            class="flex-1 flex overflow-hidden relative"
            class:select-none=move || is_dragging.get()
        >
            <div style:width=move || format!("{}%", split_width.get()) class="h-full">
                <Editor
                    contents
                    name="statement-editor"
                    readonly
                    ctrl_enter
                    on_change
                    kb_mode
                    color_mode=color_mode
                    attr:class="h-full"
                />
            </div>
            <div
                class="w-2 cursor-ew-resize hover:bg-primary transition-colors bg-base-300 active:bg-primary h-full z-10"
                on:mousedown=move |_| is_dragging.set(true)
            />
            <div style:width=move || format!("{}%", 100.0 - split_width.get()) class="h-full">
                <CompilationResults results=compilation_manager.get_result() class="h-full" />
            </div>
        </div>
    }
}
