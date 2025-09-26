use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;

use common::language::Language;
use common::statement_version::StatementVersion;
use common::task::Task;
use common::translation::{Translation, UpdateTranslationRequest};
use common::user_contest_status::SetTranslationSessionTokenRequest;
use futures::StreamExt;
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::server::codee::string::JsonSerdeCodec;
use leptos::task::{spawn_local, spawn_local_scoped};
use leptos_router::hooks::use_params_map;
use leptos_use::storage::use_local_storage;
use leptos_use::{UseColorModeOptions, signal_throttled, use_color_mode_with_options};
use thaw::{Button, Flex, Layout, LayoutHeader, Spinner};

use crate::api_wrapper::{api_get, api_post, file_get};
use crate::compilation_manager::CompilationManager;
use crate::compilation_results::CompilationResults;
use crate::edit::gemini::Gemini;
use crate::editor::{Editor, KeyboardMode};
use crate::header::Header;
use crate::session_token::get_session_token;
use crate::{show_error, show_success};

mod gemini;

const INTERVAL: u32 = 20000;

#[component]
pub fn EditPage() -> impl IntoView {
    let params = use_params_map();

    let task_id = Memo::new(move |_| {
        params
            .read()
            .get("task")
            .expect("task param")
            .parse::<i32>()
            .expect("task param should be an integer")
    });

    let lang_id = Memo::new(move |_| {
        params
            .read()
            .get("lang")
            .map(|s| s.parse::<i64>().expect("lang param should be an integer"))
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

    let lang = LocalResource::<Option<Option<Language>>>::new(move || async move {
        let Some(lang_id) = lang_id.get() else {
            return Some(None);
        };
        let url = format!("/api/languages/{}", lang_id);
        match api_get(&url).await {
            Ok(lang) => Some(lang),
            Err(e) => {
                show_error!("Failed to fetch language: {e}");
                None
            }
        }
    });

    let translation = Memo::new(move |_| {
        let task = task.read();
        let Some(Some(task)) = task.deref() else {
            return None;
        };
        let lang_id = lang_id.get();
        Some(
            task.translations
                .iter()
                .find(|t| Some(t.language_id) == lang_id)
                .cloned(),
        )
    });

    let readonly = Memo::new(move |_| {
        let translation = translation.read();
        let Some(Some(translation)) = translation.deref() else {
            return true;
        };
        translation.session_token != Some(get_session_token())
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

    let statement_path = Signal::derive(move || {
        format!(
            "{}/statement/statement.typ",
            task.get()
                .flatten()
                .map(|x| x.name.clone())
                .unwrap_or_default()
        )
    });

    let orig_text = LocalResource::<Option<String>>::new(move || async move {
        let translation = translation.read();
        let Some(translation) = translation.deref() else {
            return None;
        };
        let Some(Translation {
            content_hash: Some(hash),
            ..
        }) = translation
        else {
            let files = files.read();
            let Some(Some(files)) = files.deref() else {
                return None;
            };
            return files
                .get(&statement_path.get())
                .map(|x| String::from_utf8_lossy(x).to_string());
        };
        match file_get(hash, "statement.typ").await {
            Ok(content) => Some(String::from_utf8_lossy(&content).to_string()),
            Err(e) => {
                show_error!("Failed to fetch original statement: {e}");
                None
            }
        }
    });

    let loaded = RwSignal::new(false);

    let compilation_manager = expect_context::<CompilationManager>();

    Effect::new(move || {
        if task.get().is_some()
            && lang.get().is_some()
            && files.get().is_some()
            && orig_text.get().is_some()
        {
            loaded.set(true);
            // Compile initial state.
            compilation_manager.do_compile(true);
        } else {
            loaded.set(false);
        }
    });

    let title = Signal::derive(move || {
        let lang_code = lang
            .get()
            .flatten()
            .flatten()
            .map(|l| l.code.clone())
            .unwrap_or("en_ISC".to_owned());
        let task = task.get().flatten().map(|x| x.name).unwrap_or_default();
        format!("Task: {} - Lang: {}", task, lang_code)
    });

    let can_edit = Signal::derive(move || lang_id.get().is_some());

    let on_ask_edit = move || {
        spawn_local_scoped(async move {
            let lang = lang.await.flatten().unwrap().id;
            let task_id = task.await.unwrap().id;
            let payload = SetTranslationSessionTokenRequest {
                task_id,
                language_id: lang,
                session_token: get_session_token(),
            };
            match api_post("/api/user/set_translation_session_token", &payload).await {
                Ok(()) => {
                    show_success!("You can now edit this translation.");
                    task.refetch();
                }
                Err(e) => {
                    show_error!("Failed to set session token: {e}");
                }
            }
        });
    };

    let contents = RwSignal::new(String::new());

    Effect::new(move || {
        if let Some(text) = orig_text.get()
            && contents.with_untracked(|x| x.is_empty())
        {
            tracing::info!(ot = ?text);
            contents.set(text.unwrap_or_default());
        }
    });

    spawn_local(async move {
        loop {
            TimeoutFuture::new(INTERVAL).await;
            if loaded.try_get_untracked() != Some(true) {
                continue;
            }
            if !readonly.get_untracked() {
                continue;
            }
            let task = task.await.unwrap();
            let Some(lang) = lang.await.unwrap() else {
                continue;
            };
            let task: Task = match api_get(&format!("/api/tasks/{}", task.id)).await {
                Ok(task) => task,
                Err(e) => {
                    show_error!("Failed to fetch task: {e}");
                    continue;
                }
            };
            let translation = task
                .translations
                .into_iter()
                .find(|t| t.language_id == lang.id)
                .expect("translation should exist");
            let Some(hash) = translation.content_hash else {
                continue;
            };
            match file_get(&hash, "statement.typ").await {
                Ok(content) => {
                    let new_text = String::from_utf8_lossy(&content).to_string();
                    let res = contents.try_set(new_text);
                    if res.is_some() {
                        break;
                    }
                }
                Err(e) => {
                    show_error!("Failed to fetch original statement: {e}");
                }
            }
        }
    });

    let throttled: Signal<String> = signal_throttled(contents, 200.0);

    let compilation_manager = expect_context::<CompilationManager>();

    Effect::new(move |_| {
        throttled.with(|_| ());
        let mut files = files
            .get()
            .flatten()
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect::<HashMap<PathBuf, Signal<Vec<u8>>>>();
        files.insert(
            PathBuf::from(statement_path.get()),
            Signal::derive(move || contents.get_untracked().as_bytes().to_vec()),
        );
        compilation_manager.set_inputs(files);
    });

    let throttled: Signal<String> = signal_throttled(contents, INTERVAL as f64);
    let stored = RwSignal::new("".to_owned());

    Effect::new(move |_| {
        if loaded.try_get() != Some(true) {
            return;
        }
        if readonly.get() {
            return;
        }
        throttled.with(|_| ());
        let text = contents.get_untracked();
        spawn_local_scoped(async move {
            let task = task.await.unwrap().id;
            let Some(lang) = lang.await.unwrap() else {
                return;
            };
            let payload = UpdateTranslationRequest {
                task_id: task,
                language_id: lang.id,
                content: text.clone().into(),
                session_token: get_session_token(),
            };
            match api_post("/api/update_translation", &payload).await {
                Ok(()) => {
                    stored.set(text);
                }
                Err(e) => {
                    show_error!("Failed to auto-save translation: {e}");
                }
            }
        });
    });

    let saved = Memo::new(move |_| stored.get() == contents.get());

    let (kb_mode, set_kb_mode, _) =
        use_local_storage::<KeyboardMode, JsonSerdeCodec>("typst-translation-kb-mode");

    let compilation_manager = expect_context::<CompilationManager>();

    let color_mode = use_color_mode_with_options(
        UseColorModeOptions::default()
            .cookie_enabled(true)
            .cookie_name("typst-translation-color-mode"),
    );

    let on_change = {
        let compilation_manager = compilation_manager.clone();
        Box::new(move || compilation_manager.do_compile(false))
    };

    let ctrl_enter = {
        let compilation_manager = compilation_manager.clone();
        Box::new(move || compilation_manager.do_compile(true))
    };

    view! {
        <Spinner label="Loading statement..." class:hidden=move || loaded.get() />
        <Layout attr:style="height: 100vh" class:hidden=move || !loaded.get()>
            <LayoutHeader>
                <Header go_back="/".to_owned() title kb_mode=(kb_mode, set_kb_mode)>
                    <Show when=move || readonly.get() && can_edit.get()>
                        <Button on_click=move |_| on_ask_edit()>"Edit"</Button>
                    </Show>
                    <Show when=move || !readonly.get()>
                        <div>{move || if saved.get() { "Saved" } else { "Unsaved changes." }}</div>
                        <Gemini
                            task_id=Signal::derive(move || {
                                task.get().flatten().map(|x| x.id).unwrap_or_default()
                            })
                            lang_code=Signal::derive(move || {
                                lang
                                    .get()
                                    .flatten()
                                    .flatten()
                                    .map(|x| x.code.clone())
                                    .unwrap_or_default()
                            })
                            text=contents
                        />

                    </Show>
                </Header>
            </LayoutHeader>
            <Flex>
                <Editor
                    contents
                    name="statement-editor"
                    readonly
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
