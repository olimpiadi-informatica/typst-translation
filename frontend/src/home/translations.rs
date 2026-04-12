use common::contest::ContestWithTasksAndStatus;
use common::language::Language;
use common::task::Task;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use leptos_router::components::A;

use crate::api_wrapper::api_post;
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

    let transl_langs = all_langs
        .iter()
        .filter(|lang| lang.user_id == user_contest_status.user_id)
        .cloned()
        .collect::<Vec<_>>();

    let finalized = RwSignal::new(user_contest_status.finalized_translations);
    let dialog_ref = NodeRef::<leptos::html::Dialog>::new();

    let do_finalize = move || {
        spawn_local_scoped(async move {
            match api_post("/api/user/finalize_translation", &contest.id).await {
                Ok(()) => {
                    show_success!("Contest finalized!");
                    finalized.set(true);
                    if let Some(dialog) = dialog_ref.get() {
                        dialog.close();
                    }
                }
                Err(e) => {
                    show_error!("Failed to finalize contest: {e}");
                }
            }
        });
    };

    let open_dialog = move |_| {
        if let Some(dialog) = dialog_ref.get() {
            let _ = dialog.show_modal();
        }
    };

    let close_dialog = move |_| {
        if let Some(dialog) = dialog_ref.get() {
            dialog.close();
        }
    };

    let finalize_view = move || {
        if finalized.get() {
            Either::Left(view! {
                <button class="btn btn-primary btn-sm" disabled=true>
                    "Finalized"
                </button>
            })
        } else {
            Either::Right(view! {
                <button class="btn btn-primary btn-sm" on:click=open_dialog>
                    "Finalize"
                </button>
            })
        }
    };

    let contest_name = contest.name.clone();
    let transl_langs2 = transl_langs.clone();
    let draw_task = move |task: Task| {
        let task_id = task.id;
        let draw_edit = move |lang: Language| {
            view! {
                <td>
                    <A href=format!("/task/{}/lang/{}", task_id, lang.id)>
                        <button class="btn btn-primary btn-sm">
                            {move || if finalized.get() { "View" } else { "Edit" }}
                        </button>
                    </A>
                </td>
            }
        };

        let transl_langs2 = transl_langs2.clone();
        view! {
            <tr>
                <td>{task.name}</td>
                <For each=move || transl_langs2.clone() key=|lang| lang.id children=draw_edit />
                <td>
                    <A href=format!("/task/{}", task_id)>
                        <button class="btn btn-secondary btn-sm">"View"</button>
                    </A>
                </td>
            </tr>
        }
    };

    view! {
        <div class="card bg-base-100 shadow-none border border-base-300 mt-8">
            <div class="card-body">
                <div class="flex justify-between items-center mb-4">
                    <h2 class="card-title text-2xl font-bold">"Contest: " {contest_name}</h2>
                    {finalize_view}
                </div>
                <div class="overflow-x-auto">
                    <table class="table table-zebra w-full">
                        <thead>
                            <tr>
                                <th>"Task"</th>
                                <For each=move || transl_langs.clone() key=|lang| lang.id let(lang)>
                                    <th>"Lang " {lang.code}</th>
                                </For>
                                <th>"ISC"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For each=move || tasks.clone() key=|task| task.id children=draw_task />
                        </tbody>
                    </table>
                </div>
            </div>

            <dialog node_ref=dialog_ref class="modal">
                <div class="modal-box">
                    <h3 class="font-bold text-lg">"Finalize contest " {contest.name} "?"</h3>
                    <p class="py-4">
                        "Once the contest is finalized you will not be able to edit the statements anymore."
                    </p>
                    <div class="modal-action">
                        <button class="btn btn-primary" on:click=move |_| do_finalize()>
                            "Confirm"
                        </button>
                        <button class="btn" on:click=close_dialog>
                            "Cancel"
                        </button>
                    </div>
                </div>
                <form method="dialog" class="modal-backdrop">
                    <button>"close"</button>
                </form>
            </dialog>
        </div>
    }
}
