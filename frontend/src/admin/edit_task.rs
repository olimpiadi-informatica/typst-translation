use std::collections::HashMap;
use std::path::PathBuf;

use common::admin::UpdateTaskFilesRequest;
use common::statement_version::StatementVersion;
use common::task::Task;
use futures::StreamExt;
use leptos::ev;
use leptos::prelude::*;
use leptos::server::codee::string::JsonSerdeCodec;
use leptos::task::spawn_local_scoped;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;
use leptos_use::storage::use_local_storage;
use leptos_use::use_event_listener;

use crate::api_wrapper::{api_get, api_post, file_get};
use crate::compilation_manager::CompilationManager;
use crate::edit::layout::SplitEditorLayout;
use crate::editor::KeyboardMode;
use crate::header::Header;
use crate::util::Icon;
use crate::{show_error, show_success};

#[component]
pub fn AdminEditTaskPage() -> impl IntoView {
    let params = use_params_map();
    let task_id = Memo::new(move |_| {
        params
            .read()
            .get("task")
            .and_then(|s| s.parse::<i64>().ok())
            .expect("task param")
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

    let all_files = LocalResource::new(move || async move {
        let sv = statement_version.read();
        let Some(Some(sv)) = sv.as_ref() else {
            return None;
        };

        let futures = futures::stream::FuturesUnordered::new();
        for (key, value) in &sv.content_manifest {
            let key = key.clone();
            let value = value.clone();
            futures.push(async move {
                let name = key.rsplit('/').next().unwrap_or(&key);
                let content = file_get(&value, name).await?;
                Ok((key, content))
            });
        }

        let results: Vec<Result<(String, Vec<u8>), common::error::Error>> = futures.collect().await;
        let results: Result<HashMap<String, Vec<u8>>, common::error::Error> =
            results.into_iter().collect();

        match results {
            Ok(map) => Some(map),
            Err(e) => {
                show_error!("Failed to fetch statement files: {e}");
                None
            }
        }
    });

    let all_files_str = Memo::new(move |_| {
        all_files
            .get()
            .flatten()
            .map(|files| {
                files
                    .into_iter()
                    .map(|(k, v)| (k, String::from_utf8_lossy(&v).to_string()))
                    .collect::<HashMap<String, String>>()
            })
            .unwrap_or_default()
    });

    let selected_file = RwSignal::new(String::new());
    let overrides = RwSignal::new(HashMap::<String, String>::new());
    let current_content = RwSignal::new(String::new());
    let loading = RwSignal::new(true);

    Effect::new(move |_| {
        if let Some(Some(files)) = all_files.get() {
            if selected_file.get_untracked().is_empty() {
                let first = files
                    .keys()
                    .find(|k| k.ends_with("statement.typ"))
                    .or_else(|| files.keys().next())
                    .cloned()
                    .unwrap_or_default();
                selected_file.set(first);
            }
            loading.set(false);
        }
    });

    // When current_content changes, update overrides
    Effect::new(move |_| {
        let content = current_content.get();
        let file = selected_file.get_untracked();
        if !file.is_empty() {
            overrides.update(|o| {
                o.insert(file, content);
            });
        }
    });

    // When selected_file changes, load from overrides or all_files_str
    Effect::new(move |_| {
        let file = selected_file.get();
        if file.is_empty() {
            return;
        }
        let content = overrides
            .with_untracked(|o| o.get(&file).cloned())
            .or_else(|| all_files_str.with(|a| a.get(&file).cloned()))
            .unwrap_or_default();
        current_content.set(content);
    });

    let compilation_manager = expect_context::<CompilationManager>();

    {
        let compilation_manager = compilation_manager.clone();
        Effect::new(move |_| {
            if let Some(Some(files)) = all_files.get() {
                let mut inputs = files
                    .iter()
                    .map(|(k, v)| {
                        let v = v.clone();
                        (PathBuf::from(k), Signal::derive(move || v.clone()))
                    })
                    .collect::<HashMap<_, _>>();

                // Apply all overrides to compilation inputs
                overrides.with(|o| {
                    for (file, content) in o {
                        inputs.insert(
                            PathBuf::from(file),
                            Signal::derive({
                                let content = content.clone();
                                move || content.as_bytes().to_vec()
                            }),
                        );
                    }
                });

                compilation_manager.set_inputs(inputs);
                compilation_manager.set_extra_fonts(vec![
                    "SC".into(),
                    "TC".into(),
                    "JP".into(),
                    "KR".into(),
                ]);
                compilation_manager.do_compile(false);
            }
        });
    }

    let is_modified = Memo::new(move |_| {
        overrides.with(|o| {
            o.iter().any(|(path, content)| {
                all_files_str.with(|a| a.get(path).map(|s| s != content).unwrap_or(true))
            })
        })
    });

    let _ = use_event_listener(
        window(),
        ev::beforeunload,
        move |ev: web_sys::BeforeUnloadEvent| {
            if is_modified.get() {
                ev.prevent_default();
                ev.set_return_value("You have unsaved changes.");
            }
        },
    );

    let do_save = move || {
        spawn_local_scoped(async move {
            let task_id = task_id.get_untracked();
            let modified_files = overrides.with_untracked(|o| {
                o.iter()
                    .filter(|(path, content)| {
                        all_files_str.with(|a| a.get(*path).map(|s| s != *content).unwrap_or(true))
                    })
                    .map(|(path, content)| (path.clone(), content.as_bytes().to_vec()))
                    .collect::<Vec<_>>()
            });

            if modified_files.is_empty() {
                return;
            }

            let payload = UpdateTaskFilesRequest {
                task_id,
                files: modified_files,
            };

            match api_post("/api/admin/update_task_files", &payload).await {
                Ok(()) => {
                    show_success!("Saved all modified files");
                    overrides.set(HashMap::new());
                    statement_version.refetch();
                }
                Err(e) => {
                    show_error!("Failed to save files: {e}");
                }
            }
        });
    };

    let on_change = Box::new(move || {});
    let ctrl_enter = {
        let cm = compilation_manager.clone();
        Box::new(move || {
            cm.do_compile(true);
            do_save();
        })
    };

    let editable_files = Signal::derive(move || {
        all_files
            .get()
            .flatten()
            .map(|files| {
                let mut keys: Vec<_> = files
                    .keys()
                    .filter(|k| {
                        let k_lower = k.to_lowercase();
                        let is_text = k_lower.ends_with(".typ")
                            || k_lower.ends_with(".txt")
                            || k_lower.ends_with(".rs")
                            || k_lower.ends_with(".py")
                            || k_lower.ends_with(".cpp")
                            || k_lower.ends_with(".sh")
                            || k_lower.ends_with(".yml")
                            || k_lower.ends_with(".yaml")
                            || k_lower.ends_with(".json")
                            || k_lower.ends_with(".md")
                            || k_lower.ends_with(".toml");
                        if is_text {
                            return true;
                        }

                        let filename = std::path::Path::new(k)
                            .file_name()
                            .and_then(|f| f.to_str())
                            .unwrap_or("")
                            .to_lowercase();
                        filename == "gen"
                    })
                    .cloned()
                    .collect();
                keys.sort();
                keys
            })
            .unwrap_or_default()
    });

    let title = Signal::derive(move || {
        format!(
            "Admin Edit: {}",
            task.get()
                .flatten()
                .map(|t| t.name)
                .unwrap_or_else(|| "Loading...".to_string())
        )
    });

    let (kb_mode, _, _) =
        use_local_storage::<KeyboardMode, JsonSerdeCodec>("typst-translation-kb-mode");

    view! {
        <div class="flex justify-center items-center h-screen" class:hidden=move || !loading.get()>
            <span class="loading loading-spinner loading-lg"></span>
            <span class="ml-2">"Loading files..."</span>
        </div>
        <div class="h-screen flex flex-col" class:hidden=move || loading.get()>
            <Header
                title
                left_action=view! {
                    <A href="/admin/tasks" attr:class="btn btn-ghost btn-sm">
                        "Back to Tasks"
                    </A>
                }
                    .into_any()
            >
                <div class="flex items-center gap-4">
                    <div class="flex items-center gap-2 mr-2">
                        {move || {
                            if !is_modified.get() {
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
                                        <span class="text-sm font-medium">"Unsaved changes"</span>
                                    </div>
                                }
                            }
                        }}
                    </div>
                    <select
                        class="select select-bordered select-sm"
                        on:change=move |ev| {
                            selected_file.set(event_target_value(&ev));
                        }
                    >
                        <For each=move || editable_files.get() key=|k| k.clone() let:file_path>
                            {
                                let f1 = file_path.clone();
                                let f2 = file_path.clone();
                                let f = file_path.clone();
                                view! {
                                    <option value=f1 selected=move || { selected_file.get() == f2 }>
                                        {move || {
                                            let is_file_modified = overrides
                                                .with(|o| {
                                                    o.get(&f)
                                                        .map(|content| {
                                                            all_files_str
                                                                .with(|a| a.get(&f).map(|s| s != content).unwrap_or(true))
                                                        })
                                                        .unwrap_or(false)
                                                });
                                            if is_file_modified {
                                                format!("● {}", &f)
                                            } else {
                                                f.clone()
                                            }
                                        }}
                                    </option>
                                }
                            }
                        </For>
                    </select>
                    <button
                        class="btn btn-primary btn-sm"
                        on:click=move |_| do_save()
                        disabled=move || !is_modified.get()
                    >
                        "Save All"
                    </button>
                </div>
            </Header>
            <SplitEditorLayout
                contents=current_content
                readonly=Memo::new(|_| false)
                on_change
                ctrl_enter
                kb_mode=kb_mode
            />
        </div>
    }
}
