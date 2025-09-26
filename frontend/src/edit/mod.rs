use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;

use common::statement_version::StatementVersion;
use common::task::Task;
use futures::StreamExt;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::server::codee::string::JsonSerdeCodec;
use leptos_router::hooks::use_params_map;
use leptos_use::storage::use_local_storage;
use leptos_use::{UseColorModeOptions, use_color_mode_with_options};
use thaw::{Flex, Layout, LayoutHeader, Spinner};

use crate::api_wrapper::{api_get, file_get};
use crate::compilation_manager::CompilationManager;
use crate::compilation_results::CompilationResults;
use crate::editor::{Editor, KeyboardMode};
use crate::header::Header;
use crate::show_error;

#[component]
pub fn EditPage() -> impl IntoView {
    let task_id = Memo::new(|_| {
        let params = use_params_map();
        params
            .read()
            .get("task")
            .expect("task param")
            .parse::<i32>()
            .expect("task param should be an integer")
    });

    let task = LocalResource::<Option<Task>>::new(move || async move {
        let url = format!("/api/tasks/{}", task_id.get());
        match api_get(&url).await {
            Ok(task) => Some(task),
            Err(e) => {
                show_error!("Failed to fetch task: {e}");
                None
            }
        }
    });

    let statement_version = LocalResource::<Option<StatementVersion>>::new(move || async move {
        let url = format!("/api/tasks/{}/statement_versions/latest", task_id.get());
        match api_get(&url).await {
            Ok(files) => Some(files),
            Err(e) => {
                show_error!("Failed to fetch statement files: {e}");
                None
            }
        }
    });

    let files = LocalResource::new(move || async move {
        let statment_version = statement_version.read();
        let Some(Some(statement_version)) = statment_version.deref() else {
            return None;
        };

        let futures = futures::stream::FuturesUnordered::new();
        for (key, value) in &statement_version.content_manifest {
            let key = key.clone();
            let value = value.clone();
            futures.push(async move {
                let name = key.rsplit('/').next().unwrap_or(&key);
                let content = file_get(&value, name).await?;
                Ok((key, content))
            });
        }

        let results: Vec<Result<(String, _), Error>> = futures.collect().await;
        let results: Result<HashMap<String, _>, Error> = results.into_iter().collect();

        match results {
            Ok(map) => Some(map),
            Err(e) => {
                show_error!("Failed to fetch statement files: {e}");
                None
            }
        }
    });

    move || match (task.get().flatten(), files.get().flatten()) {
        (Some(task), Some(files)) => Either::Left(view! { <Inner task files /> }),
        _ => Either::Right(view! { <Spinner label="Loading statement..." /> }),
    }
}

#[component]
fn Inner(task: Task, files: HashMap<String, Vec<u8>>) -> impl IntoView {
    let color_mode = use_color_mode_with_options(
        UseColorModeOptions::default()
            .cookie_enabled(true)
            .cookie_name("typst-translation-color-mode"),
    );

    let (kb_mode, set_kb_mode, _) =
        use_local_storage::<KeyboardMode, JsonSerdeCodec>("typst-translation-kb-mode");

    let text_path = format!("{}/statement/statement.typ", task.name);
    let text = RwSignal::new(
        files
            .get(&text_path)
            .map(|x| String::from_utf8_lossy(x).to_string())
            .unwrap_or_default(),
    );

    let mut files = files
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect::<HashMap<PathBuf, Signal<Vec<u8>>>>();
    files.insert(
        PathBuf::from(text_path),
        Signal::derive(move || text.get().as_bytes().to_vec()),
    );
    let compilation_manager = CompilationManager::new(files);

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
