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
use leptos::ev;
use leptos::prelude::*;
use leptos::server::codee::string::JsonSerdeCodec;
use leptos::task::{spawn_local, spawn_local_scoped};
use leptos_router::hooks::{use_navigate, use_params_map};
use leptos_use::storage::use_local_storage;
use leptos_use::{ColorMode, signal_throttled, use_event_listener};

use crate::api_wrapper::{api_get, api_post, file_get};
use crate::compilation_manager::CompilationManager;
use crate::compilation_results::CompilationResults;
use crate::edit::gemini::Gemini;
use crate::edit::reset_isc::ResetIsc;
use crate::editor::{Editor, KeyboardMode};
use crate::header::Header;
use crate::session_token::get_session_token;
use crate::util::Icon;
use crate::{show_error, show_success};

mod gemini;
mod reset_isc;

const INTERVAL: u32 = 5000;

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
            Ok(lang) => Some(Some(lang)),
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

    let statement_versions =
        LocalResource::<Option<Vec<StatementVersion>>>::new(move || async move {
            let url = format!("/api/tasks/{}/statement_versions/all", task_id.get());
            match api_get(&url).await {
                Ok(files) => Some(files),
                Err(e) => {
                    show_error!("Failed to fetch statement files: {e}");
                    None
                }
            }
        });

    let old_statement_versions = Signal::derive(move || {
        statement_versions
            .get()
            .flatten()
            .unwrap_or_default()
            .into_iter()
            .skip(1)
            .collect::<Vec<_>>()
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

    let isc_version = Memo::new(move |_| {
        let files = files.read();
        let Some(Some(files)) = files.deref() else {
            return None;
        };
        files
            .get(statement_path.read().deref())
            .map(|x| String::from_utf8_lossy(x).to_string())
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
            return isc_version.get();
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

    let on_change = {
        let compilation_manager = compilation_manager.clone();
        Box::new(move || compilation_manager.do_compile(false))
    };

    let ctrl_enter = {
        let compilation_manager = compilation_manager.clone();
        Box::new(move || compilation_manager.do_compile(true))
    };

    let selected_version = RwSignal::new(String::new());

    let compare_versions = move |_| {
        let id_cur = statement_versions
            .read_untracked()
            .as_ref()
            .flatten()
            .and_then(|x| x.first().map(|x| x.id))
            .unwrap_or_default();
        let time_old = selected_version.get_untracked();
        let id_old = statement_versions
            .read_untracked()
            .as_ref()
            .flatten()
            .and_then(|x| {
                x.iter()
                    .filter_map(|x| {
                        if x.created_at.to_string() == time_old {
                            Some(x.id)
                        } else {
                            None
                        }
                    })
                    .next()
            })
            .unwrap_or_default();
        use_navigate()(
            &format!(
                "/compare/{}/{id_old}/{id_cur}",
                task.read()
                    .as_ref()
                    .flatten()
                    .map(|x| x.name.clone())
                    .unwrap_or_default()
            ),
            Default::default(),
        );
    };

    view! {
        <div class="flex justify-center items-center h-screen" class:hidden=move || loaded.get()>
            <span class="loading loading-spinner loading-lg"></span>
            <span class="ml-2">"Loading statement..."</span>
        </div>
        <div
            class="h-screen flex flex-col"
            class:hidden=move || !loaded.get()
            class:select-none=move || is_dragging.get()
        >
            <Header
                title
                kb_mode=(kb_mode, set_kb_mode)
                left_action=view! {
                    <a href="/" class="btn btn-ghost btn-sm">
                        "Home"
                    </a>
                }
                    .into_any()
            >
                <Show when=move || readonly.get() && can_edit.get()>
                    <button class="btn btn-primary btn-sm" on:click=move |_| on_ask_edit()>
                        "Edit"
                    </button>
                </Show>
                <Show when=move || !readonly.get()>
                    <div class="flex items-center gap-2">
                        {move || {
                            if saved.get() {
                                view! {
                                    <div class="flex items-center gap-1 text-success">
                                        <Icon icon=icondata::MdiContentSaveCheckOutline />
                                        <span class="text-sm font-medium">"Saved"</span>
                                    </div>
                                }
                            } else {
                                view! {
                                    <div class="flex items-center gap-1 text-warning">
                                        <Icon icon=icondata::FiLoader />
                                        <span class="text-sm font-medium">"Unsaved changes."</span>
                                    </div>
                                }
                            }
                        }}
                    </div>
                    <div class="join">
                        <Gemini
                            task_id=Signal::derive(move || {
                                task.get().flatten().map(|x| x.id).unwrap_or_default()
                            })
                            lang_code=Signal::derive(move || {
                                lang.get()
                                    .flatten()
                                    .flatten()
                                    .map(|x| x.code.clone())
                                    .unwrap_or_default()
                            })
                            text=contents
                        />
                        <ResetIsc
                            text=contents
                            isc_version=Signal::derive(move || {
                                isc_version.get().unwrap_or_default()
                            })
                        />
                    </div>
                </Show>
                <Show when=move || !old_statement_versions.read().is_empty()>
                    <div class="flex items-center gap-2">
                        <span class="text-sm">"Compare ISC versions: "</span>
                        <select
                            class="select select-bordered select-xs"
                            on:change=move |ev| {
                                selected_version.set(event_target_value(&ev));
                            }
                        >
                            <option value=""></option>
                            <For each=move || old_statement_versions.get() key=|x| x.id let:s>
                                <option value=move || {
                                    s.created_at.to_string()
                                }>{s.created_at.format("%Y-%m-%d %H:%M:%S").to_string()}</option>
                            </For>
                        </select>
                        <button class="btn btn-primary btn-xs" on:click=compare_versions>
                            "Compare"
                        </button>
                    </div>
                </Show>
            </Header>
            <div class="flex-1 flex overflow-hidden relative">
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
        </div>
    }
}
