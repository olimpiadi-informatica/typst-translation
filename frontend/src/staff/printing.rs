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
use thaw::{
    Button, Checkbox, Flex, FlexAlign, Icon, Spinner, Table, TableBody, TableCell, TableCellLayout,
    TableHeader, TableHeaderCell, TableRow,
};

use crate::api_wrapper::{api_get, file_get};
use crate::header::Header;
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
        <Header title="Printing Panel" />
        {move || match all.get().flatten() {
            Some(all) => {
                Either::Left(
                    view! {
                        <Flex vertical=true style="max-width: 800px; margin: auto">
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
                        </Flex>
                    },
                )
            }
            None => Either::Right(view! { <Spinner label="Loading..." /> }),
        }}
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
        <h2>{contest.name}</h2>
        <Table>
            <TableHeader>
                <TableRow>
                    <TableHeaderCell>"User"</TableHeaderCell>
                    <TableHeaderCell>"Finalized"</TableHeaderCell>
                </TableRow>
            </TableHeader>
            <TableBody>
                <For each=move || user_contest_status.clone() key=|ucs| ucs.id let:ucs>
                    <TableRow>
                        <TableCell>
                            <TableCellLayout>
                                {users.read().get(&ucs.user_id).unwrap().username.clone()}
                            </TableCellLayout>
                        </TableCell>
                        <TableCell>
                            <TableCellLayout>
                                {if ucs.finalized_translations {
                                    Either::Left(view! { <Icon icon=icondata::BsCheckSquare /> })
                                } else {
                                    Either::Right(())
                                }}
                            </TableCellLayout>
                        </TableCell>
                    </TableRow>
                </For>
            </TableBody>
        </Table>
        <For
            each=move || finalized_languages.get()
            key=|(lang, _contestants)| lang.as_ref().map(|l| l.id)
            let((lang, contestants))
        >
            <Flex align=FlexAlign::Center>
                <h3>{lang.as_ref().map_or_else(|| "No extra lang".into(), |l| l.code.clone())}</h3>
                <For
                    each=move || tasks.get()
                    key=move |t| t.id
                    children=move |task| {
                        let task_name = task.name.clone();
                        let lang = lang.clone();
                        view! {
                            <Button
                                icon=icondata::BsFileEarmarkPdfFill
                                on_click=move |_| do_print
                                    .read()(task.clone(), lang.as_ref().map(|l| l.id))
                            >
                                {task_name}
                            </Button>
                        }
                    }
                />
            </Flex>
            <Table>
                <For
                    each=move || contestants.clone()
                    key=|c| c.id
                    children=move |c| {
                        let local_storage = format!("printed-{}-{}", c.code, contest.id);
                        view! {
                            <TableRow>
                                <TableCell>
                                    <TableCellLayout>{c.code.clone()}</TableCellLayout>
                                </TableCell>
                                <TableCell>
                                    <TableCellLayout>
                                        <Checkbox checked={
                                            let (a, b, _) = use_local_storage::<
                                                bool,
                                                JsonSerdeCodec,
                                            >(local_storage);
                                            (a, b)
                                        } />
                                    </TableCellLayout>
                                </TableCell>
                            </TableRow>
                        }
                    }
                />
            </Table>
        </For>
    }
}
