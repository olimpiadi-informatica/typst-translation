use common::statement_version::StatementVersion;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use leptos_use::{UseColorModeOptions, use_color_mode_with_options};

use crate::api_wrapper::{api_get, file_get};
use crate::editor::DiffViewer;
use crate::header::Header;
use crate::show_error;

#[component]
pub fn Compare() -> impl IntoView {
    let params = use_params_map();

    let task_name = Memo::new(move |_| params.read().get("task_name").expect("task name"));

    let id_old = Memo::new(move |_| {
        params
            .read()
            .get("id_old")
            .expect("task param")
            .parse::<i32>()
            .expect("task param should be an integer")
    });

    let id_new = Memo::new(move |_| {
        params
            .read()
            .get("id_new")
            .expect("task param")
            .parse::<i32>()
            .expect("task param should be an integer")
    });

    let old = LocalResource::new(move || async move {
        let url = format!("/api/statement_version/{}", id_old.get());
        match api_get::<StatementVersion>(&url).await {
            Ok(files) => Some(files),
            Err(e) => {
                show_error!("Failed to fetch statement files: {e}");
                None
            }
        }
    });

    let new = LocalResource::new(move || async move {
        let url = format!("/api/statement_version/{}", id_new.get());
        match api_get::<StatementVersion>(&url).await {
            Ok(files) => Some(files),
            Err(e) => {
                show_error!("Failed to fetch statement files: {e}");
                None
            }
        }
    });

    let loaded = RwSignal::new(false);

    Effect::new(move || {
        if new.get().is_some() && old.get().is_some() {
            loaded.set(true);
        } else {
            loaded.set(false);
        }
    });

    let statement_path =
        Signal::derive(move || format!("{}/statement/statement.typ", task_name.get()));

    let statements = LocalResource::new(move || async move {
        let old_statement = old
            .await
            .as_ref()
            .and_then(|x| x.content_manifest.get(&*statement_path.read()))
            .cloned()
            .unwrap_or_default();
        let new_statement = new
            .await
            .as_ref()
            .and_then(|x| x.content_manifest.get(&*statement_path.read()))
            .cloned()
            .unwrap_or_default();

        let statements = async {
            Ok::<_, Error>((
                file_get(&old_statement, "statement.typ").await?,
                file_get(&new_statement, "statement.typ").await?,
            ))
        }
        .await;

        match statements {
            Ok(map) => Some(map),
            Err(e) => {
                show_error!("Failed to fetch statement files: {e}");
                None
            }
        }
    });

    let first = Signal::derive(move || {
        String::from_utf8_lossy(
            &statements
                .read()
                .as_ref()
                .flatten()
                .map(|x| x.0.clone())
                .unwrap_or_default(),
        )
        .to_string()
    });

    let second = Signal::derive(move || {
        String::from_utf8_lossy(
            &statements
                .read()
                .as_ref()
                .flatten()
                .map(|x| x.1.clone())
                .unwrap_or_default(),
        )
        .to_string()
    });

    let color_mode = use_color_mode_with_options(
        UseColorModeOptions::default()
            .cookie_enabled(true)
            .cookie_name("typst-translation-color-mode"),
    )
    .mode;

    let go_back = move |_| {
        if let Some(window) = web_sys::window()
            && let Ok(history) = window.history()
        {
            let _ = history.back();
        }
    };

    view! {
        <div
            class="flex justify-center items-center h-screen"
            class:hidden=move || loaded.get()
        >
            <span class="loading loading-spinner loading-lg"></span>
            <span class="ml-2">"Loading..."</span>
        </div>
        <div class="h-screen flex flex-col" class:hidden=move || !loaded.get()>
            <Header title=Signal::derive(move || {
                format!("Comparing ISC versions for task {}", task_name.get())
            }) left_action=view! {
                <button class="btn btn-ghost btn-sm gap-2" on:click=go_back>
                    "Back to Editing"
                </button>
            }.into_any()></Header>
            <div class="flex-1 overflow-hidden">
                <DiffViewer
                    color_mode
                    first
                    second
                    name="isc"
                    attr:class="w-full h-full"
                ></DiffViewer>
            </div>
        </div>
    }
}
