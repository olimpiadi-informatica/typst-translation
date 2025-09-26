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
use leptos::either::{Either, EitherOf3};
use leptos::prelude::*;
use leptos::server::codee::string::JsonSerdeCodec;
use leptos::task::{spawn_local, spawn_local_scoped};
use leptos_router::hooks::use_params_map;
use leptos_use::storage::use_local_storage;
use leptos_use::{UseColorModeOptions, signal_throttled, use_color_mode_with_options};
use thaw::{Button, Flex, FlexAlign, Layout, LayoutHeader, Spinner};

use crate::api_wrapper::{api_get, api_post, file_get};
use crate::app::wrap_with_current_owner;
use crate::compilation_manager::CompilationManager;
use crate::compilation_results::CompilationResults;
use crate::edit::gemini::Gemini;
use crate::editor::{Editor, KeyboardMode};
use crate::header::Header;
use crate::session_token::get_session_token;
use crate::user::UserContext;
use crate::{show_error, show_success};

mod gemini;

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
            let task = task.read();
            let Some(Some(task)) = task.deref() else {
                return None;
            };
            let files = files.read();
            let Some(Some(files)) = files.deref() else {
                return None;
            };
            return files
                .get(&format!("{}/statement/statement.typ", task.name))
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

    move || match (
        task.get().flatten(),
        lang.get().flatten(),
        files.get().flatten(),
        orig_text.get().flatten(),
    ) {
        (Some(task_val), Some(lang), Some(files), Some(orig_text)) => {
            let readonly = readonly.get();
            Either::Left(
                view! { <Inner task_resource=task task=task_val lang files readonly orig_text /> },
            )
        }
        _ => Either::Right(view! { <Spinner label="Loading statement..." /> }),
    }
}

#[component]
fn Inner(
    task_resource: LocalResource<Option<Task>>,
    task: Task,
    lang: Option<Language>,
    files: HashMap<String, Vec<u8>>,
    readonly: bool,
    orig_text: String,
) -> impl IntoView {
    let color_mode = use_color_mode_with_options(
        UseColorModeOptions::default()
            .cookie_enabled(true)
            .cookie_name("typst-translation-color-mode"),
    );

    let (kb_mode, set_kb_mode, _) =
        use_local_storage::<KeyboardMode, JsonSerdeCodec>("typst-translation-kb-mode");

    let text_path = format!("{}/statement/statement.typ", task.name);
    let text = RwSignal::new(orig_text);

    let mut saved_view = None;

    const INTERVAL: u32 = 20000;
    if readonly {
        if let Some(Language { id: lang_id, .. }) = lang {
            let task = task.clone();
            spawn_local(async move {
                loop {
                    TimeoutFuture::new(INTERVAL).await;
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
                        .find(|t| t.language_id == lang_id)
                        .expect("translation should exist");
                    let Some(hash) = translation.content_hash else {
                        continue;
                    };
                    match file_get(&hash, "statement.typ").await {
                        Ok(content) => {
                            let new_text = String::from_utf8_lossy(&content).to_string();
                            let res = text.try_set(new_text);
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
        }
    } else {
        let throttled: Signal<String> = signal_throttled(text, INTERVAL as f64);
        let stored = RwSignal::new("".to_owned());

        let lang_id = lang.as_ref().unwrap().id;
        Effect::new(move |_| {
            let text = throttled.get();
            spawn_local_scoped(async move {
                let payload = UpdateTranslationRequest {
                    task_id: task.id,
                    language_id: lang_id,
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

        let saved = Memo::new(move |_| stored.get() == text.get());
        saved_view = Some(
            view! { <div>{move || if saved.get() { "Saved" } else { "Unsaved changes." }}</div> },
        );
    }

    let mut files = files
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect::<HashMap<PathBuf, Signal<Vec<u8>>>>();
    files.insert(
        PathBuf::from(text_path),
        Signal::derive(move || text.get().as_bytes().to_vec()),
    );
    let compilation_manager = expect_context::<CompilationManager>();
    compilation_manager.set_inputs(files);

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

    let user_context = expect_context::<UserContext>();
    let user = user_context.get_user_untracked();
    let user_id = user.as_user().unwrap().id;

    let lang2 = lang.clone();
    let do_set_token = wrap_with_current_owner(move || {
        let lang2 = lang2.clone();
        spawn_local_scoped(async move {
            let payload = SetTranslationSessionTokenRequest {
                task_id: task.id,
                language_id: lang2.unwrap().id,
                session_token: get_session_token(),
            };
            match api_post("/api/user/set_translation_session_token", &payload).await {
                Ok(()) => {
                    show_success!("You can now edit this translation.");
                    task_resource.refetch();
                }
                Err(e) => {
                    show_error!("Failed to set session token: {e}");
                }
            }
        });
    });

    let lang_code = lang
        .as_ref()
        .map(|l| l.code.clone())
        .unwrap_or("en_ISC".to_owned());
    view! {
        <Layout attr:style="height: 100vh">
            <LayoutHeader>
                <Header
                    go_back="/".to_owned()
                    title=format!("Task: {} - Lang: {}", task.name, lang_code)
                    kb_mode=(kb_mode, set_kb_mode)
                >
                    <Flex align=FlexAlign::Center>
                        {if lang.as_ref().map(|l| l.user_id) == Some(user_id) {
                            if readonly {
                                EitherOf3::A(
                                    view! {
                                        <Button on_click=move |_| do_set_token()>"Edit"</Button>
                                    },
                                )
                            } else {
                                EitherOf3::B(
                                    view! {
                                        {saved_view}
                                        <Gemini
                                            task_id=task.id
                                            lang_code=lang.as_ref().unwrap().code.clone()
                                            text
                                        />
                                    },
                                )
                            }
                        } else {
                            EitherOf3::C(())
                        }}
                    </Flex>
                </Header>
            </LayoutHeader>
            <Flex>
                <Editor
                    contents=text
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
