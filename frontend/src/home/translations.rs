use common::contest::ContestWithTasksAndStatus;
use common::language::Language;
use common::task::Task;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use thaw::{
    Button, ButtonAppearance, Dialog, DialogActions, DialogBody, DialogContent, DialogSurface,
    DialogTitle, Flex, FlexJustify, Table, TableBody, TableCell, TableCellLayout, TableHeader,
    TableHeaderCell, TableRow,
};

use crate::api_wrapper::api_post;
use crate::app::wrap_with_current_owner;
use crate::{show_error, show_success};

#[component]
pub fn Translations(
    contests: Vec<ContestWithTasksAndStatus>,
    all_langs: Vec<Language>,
) -> impl IntoView {
    contests
        .into_iter()
        .map(move |contest| view! { <Contest contest all_langs=all_langs.clone() /> })
        .collect::<Vec<_>>()
}

#[component]
fn Contest(contest: ContestWithTasksAndStatus, all_langs: Vec<Language>) -> impl IntoView {
    let ContestWithTasksAndStatus {
        contest,
        tasks,
        user_contest_status,
    } = contest;

    let finalized = RwSignal::new(user_contest_status.finalized_translations);
    let open_finalize_dialog = RwSignal::new(false);
    let do_finalize = wrap_with_current_owner(move || {
        spawn_local_scoped(async move {
            match api_post("/api/user/finalize_translation", &contest.id).await {
                Ok(()) => {
                    show_success!("Contest finalized!");
                    finalized.set(true);
                    open_finalize_dialog.set(false);
                }
                Err(e) => {
                    show_error!("Failed to finalize contest: {e}");
                }
            }
        })
    });
    let finalize_view = move || {
        if finalized.get() {
            Either::Left(view! {
                // TODO: something better than a disabled button?
                <Button appearance=ButtonAppearance::Primary disabled=true>
                    "Finalized"
                </Button>
            })
        } else {
            Either::Right(
                view! { <Button on_click=move |_| open_finalize_dialog.set(true)>"Finalize"</Button> },
            )
        }
    };

    let name = contest.name.clone();

    let all_langs2 = all_langs.clone();
    let draw_task = move |task: Task| {
        let task_name = task.name.clone();
        let draw_edit = move |lang: Language| {
            let task_name = task_name.clone();
            view! {
                <TableCell>
                    <TableCellLayout>
                        <a href=format!("/edit/task/{}/lang/{}", task_name.clone(), lang.code)>
                            <Button>"Edit"</Button>
                        </a>
                    </TableCellLayout>
                </TableCell>
            }
        };

        let all_langs2 = all_langs2.clone();
        let task_name = task.name.clone();
        view! {
            <TableRow>
                <TableCell>
                    <TableCellLayout>{task.name}</TableCellLayout>
                </TableCell>
                <For each=move || all_langs2.clone() key=|lang| lang.id children=draw_edit />
                <TableCell>
                    <TableCellLayout>
                        <a href=format!("/edit/task/{}/lang/en_ISC", task_name)>
                            <Button>"View"</Button>
                        </a>
                    </TableCellLayout>
                </TableCell>
            </TableRow>
        }
    };

    view! {
        <Flex vertical=true>
            <Flex justify=FlexJustify::SpaceBetween>
                <h1>"Contest: " {name}</h1>
                {finalize_view}
            </Flex>
            <Table>
                <TableHeader>
                    <TableRow>
                        <TableHeaderCell>"Task"</TableHeaderCell>
                        <For each=move || all_langs.clone() key=|lang| lang.id let(lang)>
                            <TableHeaderCell>"Lang " {lang.code}</TableHeaderCell>
                        </For>
                        <TableHeaderCell>"ISC"</TableHeaderCell>
                    </TableRow>
                </TableHeader>
                <TableBody>
                    <For each=move || tasks.clone() key=|task| task.id children=draw_task />
                </TableBody>
            </Table>
        </Flex>

        <Dialog open=open_finalize_dialog>
            <DialogSurface>
                <DialogBody>
                    <DialogTitle>"Finalize contest " {contest.name} "?"</DialogTitle>
                    <DialogContent>
                        "Once the contest is finalized you will not be able to edit the statements anymore."
                    </DialogContent>
                    <DialogActions>
                        <Button
                            appearance=ButtonAppearance::Primary
                            on_click=move |_| do_finalize()
                        >
                            "Confirm"
                        </Button>
                    </DialogActions>
                </DialogBody>
            </DialogSurface>
        </Dialog>
    }
}
