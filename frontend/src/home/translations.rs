use common::contest::ContestWithTasksAndStatus;
use common::language::Language;
use common::task::Task;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use thaw::{
    Button, ButtonAppearance, Dialog, DialogActions, DialogBody, DialogContent, DialogSurface,
    DialogTitle, Flex, FlexJustify,
};

use crate::api_wrapper::api_post;
use crate::app::wrap_with_current_owner;
use crate::{show_error, show_success};

#[component]
pub fn Translations(
    contests: Vec<ContestWithTasksAndStatus>,
    all_langs: Vec<Language>,
) -> impl IntoView {
    view! {
        <h1>"Translations"</h1>
        {contests
            .into_iter()
            .map(move |contest| view! { <Contest contest all_langs=all_langs.clone() /> })
            .collect::<Vec<_>>()}
    }
}

#[component]
fn Contest(contest: ContestWithTasksAndStatus, all_langs: Vec<Language>) -> impl IntoView {
    let rows = contest
        .tasks
        .into_iter()
        .map(|task| {
            view! { <Task task all_langs=all_langs.clone() /> }
        })
        .collect::<Vec<_>>();

    let finalized = RwSignal::new(contest.user_contest_status.finalized_translations);
    let open_finalize_dialog = RwSignal::new(false);

    let do_finalize = wrap_with_current_owner(move || {
        spawn_local_scoped(async move {
            match api_post("/api/user/finalize_translation", &contest.contest.id).await {
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

    let name = contest.contest.name.clone();

    view! {
        <Flex
            vertical=true
            style="margin-bottom: 2em; padding: 1em; border: 2px solid #000; border-radius: 6px;"
        >
            <Flex justify=FlexJustify::SpaceBetween>
                <h2>"Contest: " {name}</h2>
                {move || {
                    if finalized.get() {
                        Either::Left(
                            view! {
                                // TODO: something better than a disabled button?
                                <Button appearance=ButtonAppearance::Primary disabled=true>"Finalized"</Button>
                            },
                        )
                    } else {
                        Either::Right(
                            view! {
                                <Button on_click=move |_| {
                                    open_finalize_dialog.set(true)
                                }>"Finalize"</Button>
                            },
                        )
                    }
                }}
            </Flex>
            {rows}
        </Flex>

        <Dialog open=open_finalize_dialog>
            <DialogSurface>
                <DialogBody>
                    <DialogTitle>"Finalize contest " {contest.contest.name} "?"</DialogTitle>
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

#[component]
fn Task(task: Task, all_langs: Vec<Language>) -> impl IntoView {
    let langs = task
        .translations
        .into_iter()
        .map(move |transl| {
            let code = all_langs
                .iter()
                .find(|lang| lang.id == transl.language_id)
                .map(|lang| lang.code.clone())
                .unwrap_or_default();
            view! { {code} }
        })
        .collect::<Vec<_>>();

    view! {
        <Flex
            vertical=true
            style="margin-bottom: 1em; padding: 0.5em; border: 1px solid #ccc; border-radius: 4px;"
        >
            <p>"Task: " {task.name}</p>
            <div style="margin-left: 1em;">{langs}</div>
        </Flex>
    }
}
