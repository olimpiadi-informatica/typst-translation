use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use common::contest::{All, ContestWithAll};
use common::contestant::Contestant;
use common::language::Language;
use common::statement_version::StatementVersion;
use common::task::Task;
use common::translation::Translation;
use common::user::User;
use futures::StreamExt;
use gloo_worker::Spawnable;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::server::codee::string::JsonSerdeCodec;
use leptos::task::spawn_local_scoped;
use leptos_use::storage::use_local_storage;

use crate::api_wrapper::{api_get, file_get};
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
                                    <Contest
                                        contest
                                        contestants=all.contestants.clone()
                                        languages=all.languages.clone()
                                        users=all.users.clone()
                                    />
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
pub fn Contest(
    contest: ContestWithAll,
    contestants: Vec<Contestant>,
    languages: Vec<Language>,
    users: Vec<User>,
) -> impl IntoView {
    let ContestWithAll {
        contest,
        user_contest_status,
        tasks,
    } = contest;

    let tasks = Signal::stored(tasks);

    let finalized_users = Signal::stored(
        user_contest_status
            .iter()
            .filter(|ucs| ucs.finalized_translations)
            .map(|ucs| ucs.user_id)
            .collect::<HashSet<_>>(),
    );

    let users = Signal::stored(
        users
            .into_iter()
            .map(|u| (u.id, u))
            .collect::<HashMap<_, _>>(),
    );

    let finalized_contestants = Signal::stored(
        contestants
            .into_iter()
            .filter(|c| finalized_users.read().contains(&c.user_id))
            .collect::<Vec<_>>(),
    );

    let finalized_languages = Signal::stored(
        languages
            .into_iter()
            .filter(|lang| finalized_users.read().contains(&lang.user_id))
            .map(Some)
            .chain(std::iter::once(None))
            .map(|lang| {
                let lang_id = lang.as_ref().map(|l| l.id);
                (
                    lang,
                    finalized_contestants
                        .read()
                        .iter()
                        .filter(|c| c.language_id == lang_id)
                        .cloned()
                        .collect::<Vec<_>>(),
                )
            })
            .filter(|(_, contestants)| !contestants.is_empty())
            .collect::<Vec<_>>(),
    );

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
            <h2 class="text-3xl font-bold border-b-2 border-primary pb-2">{contest.name}</h2>

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
                            <For each=move || user_contest_status.clone() key=|ucs| ucs.id let:ucs>
                                <tr>
                                    <td>
                                        {users.read().get(&ucs.user_id).unwrap().username.clone()}
                                    </td>
                                    <td>
                                        {if ucs.finalized_translations {
                                            Either::Left(
                                                view! {
                                                    <span class="text-success">
                                                        <Icon icon=icondata::BsCheckSquare />
                                                    </span>
                                                },
                                            )
                                        } else {
                                            Either::Right(())
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
                                            let local_storage = format!(
                                                "printed-{}-{}",
                                                c.code,
                                                contest.id,
                                            );
                                            let (checked, set_checked, _) = use_local_storage::<
                                                bool,
                                                JsonSerdeCodec,
                                            >(local_storage);
                                            view! {
                                                <tr>
                                                    <td>{c.code.clone()}</td>
                                                    <td>
                                                        <input
                                                            type="checkbox"
                                                            class="checkbox checkbox-primary"
                                                            checked=checked
                                                            on:change=move |ev| {
                                                                set_checked.set(event_target_checked(&ev));
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
