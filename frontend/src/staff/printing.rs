use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use common::admin::UpdateContestantPrintStatusRequest;
use common::contest::All;
use common::statement_version::StatementVersion;
use common::task::Task;
use common::translation::Translation;
use futures::StreamExt;
use gloo_worker::Spawnable;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;

use crate::api_wrapper::{api_get, api_post, file_get};
use crate::util::Icon;
use crate::{TypstWorker, show_error};

#[component]
pub fn PrintingPage() -> impl IntoView {
    let all = LocalResource::<Option<All>>::new(|| async {
        match api_get("/api/all").await {
            Ok(data) => Some(data),
            Err(e) => {
                show_error!("Failed to fetch translation status: {}", e);
                None
            }
        }
    });
    provide_context(all);

    view! {
        <div class="container mx-auto max-w-4xl p-4">
            {move || match all.get().flatten() {
                Some(all) => {
                    Either::Left(
                        view! {
                            <div class="flex flex-col gap-12">
                                <For
                                    each=move || all.contests.clone()
                                    key=|contest| contest.contest.id
                                    let:contest
                                >
                                    <Contest contest_id=contest.contest.id />
                                </For>
                            </div>
                        },
                    )
                }
                None => {
                    Either::Right(
                        view! {
                            <div class="flex flex-col items-center py-12">
                                <span class="loading loading-spinner loading-lg text-primary"></span>
                                <p class="mt-4 text-base-content/60">"Loading..."</p>
                            </div>
                        },
                    )
                }
            }}
        </div>
    }
}

#[component]
pub fn Contest(contest_id: i64) -> impl IntoView {
    let all = use_context::<LocalResource<Option<All>>>().expect("all resource not provided");

    let contest_with_all = Signal::derive(move || {
        all.get().flatten().and_then(|all| {
            all.contests
                .into_iter()
                .find(|c| c.contest.id == contest_id)
        })
    });

    let contest = Signal::derive(move || contest_with_all.get().map(|c| c.contest));
    let user_contest_status =
        Signal::derive(move || contest_with_all.get().map(|c| c.user_contest_status));
    let tasks = Signal::derive(move || contest_with_all.get().map(|c| c.tasks).unwrap_or_default());
    let printed_contestants = Signal::derive(move || {
        contest_with_all
            .get()
            .map(|c| c.printed_contestants.into_iter().collect::<HashSet<_>>())
            .unwrap_or_default()
    });

    let toggle_printed = move |contestant_id: i64, printed: bool| {
        spawn_local_scoped(async move {
            let payload = UpdateContestantPrintStatusRequest {
                contest_id,
                contestant_id,
                printed,
            };
            match api_post("/api/admin/contest/contestant/print_status", &payload).await {
                Ok(()) => {
                    all.refetch();
                }
                Err(e) => {
                    show_error!("Failed to update print status: {}", e);
                }
            }
        });
    };

    let user_status_map = Signal::derive(move || {
        user_contest_status
            .get()
            .unwrap_or_default()
            .into_iter()
            .map(|ucs| (ucs.user_id, ucs))
            .collect::<HashMap<_, _>>()
    });

    let finalized_users = Signal::derive(move || {
        user_contest_status
            .get()
            .unwrap_or_default()
            .iter()
            .filter(|ucs| ucs.finalized_translations)
            .map(|ucs| ucs.user_id)
            .collect::<HashSet<_>>()
    });

    let users = Signal::derive(move || {
        all.get()
            .flatten()
            .map(|all| {
                all.users
                    .into_iter()
                    .map(|u| (u.id, u))
                    .collect::<HashMap<_, _>>()
            })
            .unwrap_or_default()
    });

    let all_contestants = Signal::derive(move || {
        all.get()
            .flatten()
            .map(|all| all.contestants)
            .unwrap_or_default()
    });

    let languages = Signal::derive(move || {
        all.get()
            .flatten()
            .map(|all| all.languages)
            .unwrap_or_default()
    });

    let finalized_contestants = Signal::derive(move || {
        let finalized = finalized_users.get();
        all_contestants
            .get()
            .into_iter()
            .filter(|c| finalized.contains(&c.user_id))
            .collect::<Vec<_>>()
    });

    let queue_items = Signal::derive(move || {
        let printed = printed_contestants.get();
        let finalized = finalized_users.get();
        let statuses = user_status_map.get();

        let mut items: Vec<_> = all_contestants
            .get()
            .into_iter()
            .filter(|c| finalized.contains(&c.user_id) && !printed.contains(&c.id))
            .collect();

        items.sort_by_key(|c| statuses.get(&c.user_id).and_then(|s| s.finalized_at));
        items
    });

    let finalized_languages = Signal::derive(move || {
        let finalized = finalized_users.get();
        let fc = finalized_contestants.get();
        languages
            .get()
            .into_iter()
            .filter(|lang| finalized.contains(&lang.user_id))
            .map(Some)
            .chain(std::iter::once(None))
            .map(|lang| {
                let lang_id = lang.as_ref().map(|l| l.id);
                (
                    lang,
                    fc.iter()
                        .filter(|c| c.language_id == lang_id)
                        .cloned()
                        .collect::<Vec<_>>(),
                )
            })
            .filter(|(_, contestants)| !contestants.is_empty())
            .collect::<Vec<_>>()
    });

    let owner = Owner::new();
    let do_print = move |task: Task, lang_id: Option<i64>| {
        owner.with(move || {
            spawn_local_scoped(async move {
                let url = format!("/api/tasks/{}/statement_versions/latest", task.id);
                let statement: StatementVersion = match api_get(&url).await {
                    Ok(statements) => statements,
                    Err(e) => {
                        show_error!("Failed to fetch statement version: {}", e);
                        return;
                    }
                };

                let futures = futures::stream::FuturesUnordered::new();
                for (key, value) in &statement.content_manifest {
                    let key = key.clone();
                    let value = value.clone();
                    futures.push(async move {
                        let name = key.rsplit('/').next().unwrap_or(&key);
                        let content = file_get(&value, name).await?;
                        Ok((key.into(), content))
                    });
                }

                let files: Vec<Result<(PathBuf, _), Error>> = futures.collect().await;
                let files: Result<HashMap<PathBuf, _>, Error> = files.into_iter().collect();

                let mut files = match files {
                    Ok(files) => files,
                    Err(e) => {
                        show_error!("Failed to fetch statement files: {e}");
                        return;
                    }
                };

                if let Some(lang_id) = lang_id {
                    let translation = task
                        .translations
                        .into_iter()
                        .find(|t| t.language_id == lang_id);
                    let Some(Translation {
                        content_hash: Some(hash),
                        ..
                    }) = translation
                    else {
                        show_error!("Untranslated statement!");
                        return;
                    };

                    let statement = match file_get(&hash, "statement.typ").await {
                        Ok(statement) => statement,
                        Err(e) => {
                            show_error!("Failed to fetch statement translation: {e}");
                            return;
                        }
                    };

                    files.insert(
                        PathBuf::from(format!("{}/statement/statement.typ", task.name)),
                        statement,
                    );
                }

                let mut typst_worker =
                    TypstWorker::spawner().spawn_with_loader("/typst_translation_worker_loader.js");
                typst_worker.send_input(files);
                let response = typst_worker.next().await.unwrap();

                let document = match response.document {
                    Some(doc) => doc,
                    None => {
                        show_error!("Failed to compile statement translation: no output");
                        return;
                    }
                };

                let array8 = js_sys::Uint8Array::from(document.pdf.as_slice());
                let array = js_sys::Array::of1(&array8);
                let opts = web_sys::BlobPropertyBag::new();
                opts.set_type("application/pdf");
                let blob =
                    web_sys::Blob::new_with_u8_array_sequence_and_options(&array, &opts).unwrap();

                let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
                web_sys::window()
                    .unwrap()
                    .open_with_url(&url)
                    .unwrap_or(None);
            });
        });
    };

    let do_print = Signal::stored(do_print);

    view! {
        <div class="flex flex-col gap-6">
            <h2 class="text-3xl font-bold border-b-2 border-primary pb-2">
                {move || contest.get().map(|c| c.name).unwrap_or_default()}
            </h2>

            <div class="card bg-base-200 shadow-xl border-l-4 border-accent">
                <div class="card-body">
                    <h3 class="card-title text-2xl flex items-center gap-2">
                        <Icon icon=icondata::BsPrinterFill />
                        "Print Queue"
                        <span class="badge badge-accent">{move || queue_items.get().len()}</span>
                    </h3>
                    <div class="overflow-x-auto">
                        <table class="table w-full">
                            <thead>
                                <tr>
                                    <th>"Contestant"</th>
                                    <th>"Language"</th>
                                    <th>"Tasks"</th>
                                    <th>"Action"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <For each=move || queue_items.get() key=|c| c.id let:c>
                                    {
                                        let user_lang = Signal::derive(move || {
                                            languages.get().into_iter().find(|l| l.user_id == c.user_id)
                                        });
                                        let lang_name = move || {
                                            user_lang
                                                .get()
                                                .map(|l| l.code.clone())
                                                .unwrap_or_else(|| "Original".into())
                                        };
                                        let lang_id = move || user_lang.get().map(|l| l.id);
                                        view! {
                                            <tr>
                                                <td>
                                                    <div class="font-bold">{c.code.clone()}</div>
                                                    <div class="text-sm opacity-50">{c.name.clone()}</div>
                                                </td>
                                                <td>
                                                    <span class="badge badge-outline">{lang_name}</span>
                                                </td>
                                                <td>
                                                    <div class="flex gap-1">
                                                        <For each=move || tasks.get() key=|t| t.id let:task>
                                                            <button
                                                                class="btn btn-ghost btn-xs btn-square tooltip"
                                                                data-tip=task.name.clone()
                                                                on:click=move |_| do_print.read()(task.clone(), lang_id())
                                                            >
                                                                <Icon icon=icondata::BsFileEarmarkPdfFill />
                                                            </button>
                                                        </For>
                                                    </div>
                                                </td>
                                                <td>
                                                    <button
                                                        class="btn btn-accent btn-sm"
                                                        on:click=move |_| toggle_printed(c.id, true)
                                                    >
                                                        "Mark Printed"
                                                    </button>
                                                </td>
                                            </tr>
                                        }
                                    }
                                </For>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>

            <div class="card bg-base-100 shadow-xl">
                <div class="card-body p-0 overflow-x-auto">
                    <table class="table table-zebra w-full">
                        <thead>
                            <tr>
                                <th>"User"</th>
                                <th>"Finalized"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || user_contest_status.get().unwrap_or_default()
                                key=|ucs| ucs.id
                                let:ucs
                            >
                                <tr>
                                    <td>
                                        {move || {
                                            users
                                                .get()
                                                .get(&ucs.user_id)
                                                .map(|u| u.username.clone())
                                                .unwrap_or_default()
                                        }}
                                    </td>
                                    <td>
                                        {move || {
                                            if ucs.finalized_translations {
                                                Either::Left(
                                                    view! {
                                                        <span class="text-success">
                                                            <Icon icon=icondata::BsCheckSquare />
                                                        </span>
                                                    },
                                                )
                                            } else {
                                                Either::Right(())
                                            }
                                        }}
                                    </td>
                                </tr>
                            </For>
                        </tbody>
                    </table>
                </div>
            </div>

            <For
                each=move || finalized_languages.get()
                key=|(lang, _contestants)| lang.as_ref().map(|l| l.id)
                let((lang, contestants))
            >
                <div class="card bg-base-100 shadow-md">
                    <div class="card-body">
                        <div class="flex justify-between items-center mb-4">
                            <h3 class="card-title text-xl">
                                {lang
                                    .as_ref()
                                    .map_or_else(|| "No extra lang".into(), |l| l.code.clone())}
                            </h3>
                            <div class="flex flex-wrap gap-2">
                                <For
                                    each=move || tasks.get()
                                    key=move |t| t.id
                                    children=move |task| {
                                        let task_name = task.name.clone();
                                        let lang = lang.clone();
                                        view! {
                                            <button
                                                class="btn btn-outline btn-primary btn-sm gap-2"
                                                on:click=move |_| do_print
                                                    .read()(task.clone(), lang.as_ref().map(|l| l.id))
                                            >
                                                <Icon icon=icondata::BsFileEarmarkPdfFill />
                                                {task_name}
                                            </button>
                                        }
                                    }
                                />
                            </div>
                        </div>

                        <div class="overflow-x-auto">
                            <table class="table table-sm w-full">
                                <thead>
                                    <tr>
                                        <th>"Code"</th>
                                        <th>"Printed"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <For
                                        each=move || contestants.clone()
                                        key=|c| c.id
                                        children=move |c| {
                                            view! {
                                                <tr>
                                                    <td>{c.code.clone()}</td>
                                                    <td>
                                                        <input
                                                            type="checkbox"
                                                            class="checkbox checkbox-primary"
                                                            checked=move || printed_contestants.get().contains(&c.id)
                                                            on:change=move |ev| {
                                                                toggle_printed(c.id, event_target_checked(&ev));
                                                            }
                                                        />
                                                    </td>
                                                </tr>
                                            }
                                        }
                                    />
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </For>
        </div>
    }
}
