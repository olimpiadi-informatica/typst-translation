use common::admin::CreateContestRequest;
use common::contest::All;
use common::translation::ImportTaskPayload;
use js_sys::Uint8Array;
use leptos::either::Either;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use leptos_router::components::A;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use crate::api_wrapper::{api_get, api_post};
use crate::{show_error, show_success};

#[component]
pub fn AdminTasksPage() -> impl IntoView {
    let all_data = LocalResource::<Option<All>>::new(move || async move {
        match api_get("/api/all").await {
            Ok(all) => Some(all),
            Err(e) => {
                show_error!("Failed to fetch tasks: {e}");
                None
            }
        }
    });

    let input_ref: NodeRef<html::Input> = NodeRef::new();
    let contest_id_signal = RwSignal::new(String::new());
    let update_signal = RwSignal::new(false);
    let loading = RwSignal::new(false);

    let do_import = move || {
        spawn_local_scoped(async move {
            loading.set(true);

            let contest_id = contest_id_signal
                .get_untracked()
                .parse::<i64>()
                .expect("Invalid contest ID");
            let update_val = update_signal.get_untracked();
            let files = input_ref
                .get_untracked()
                .and_then(|input| input.files())
                .expect("No files selected");

            for idx in 0..files.length() {
                let file = files.get(idx).expect("Failed to get file from FileList");
                let file_name = file.name();
                let buffer: Uint8Array =
                    JsFuture::from(file.bytes()).await.unwrap().unchecked_into();
                let zip_file = buffer.to_vec();

                let payload = ImportTaskPayload {
                    contest_id,
                    update: update_val,
                    zip_file,
                };
                match api_post("/api/tasks/import", &payload).await {
                    Ok(()) => {
                        show_success!("Imported file: {}", file_name);
                        all_data.refetch();
                    }
                    Err(e) => {
                        show_error!("Failed to import file {}: {}", file_name, e);
                    }
                }
            }

            loading.set(false);
        });
    };

    let new_contest_name = RwSignal::new(String::new());
    let contest_loading = RwSignal::new(false);

    let do_create_contest = move || {
        spawn_local_scoped(async move {
            contest_loading.set(true);
            let payload = CreateContestRequest {
                name: new_contest_name.get_untracked(),
            };
            match api_post("/api/admin/create_contest", &payload).await {
                Ok(()) => {
                    show_success!("Created contest: {}", payload.name);
                    new_contest_name.set(String::new());
                    all_data.refetch();
                }
                Err(e) => {
                    show_error!("Failed to create contest: {e}");
                }
            }
            contest_loading.set(false);
        });
    };

    view! {
        <div class="p-8 flex flex-col gap-8">
            <div class="flex flex-col md:flex-row gap-8 items-start justify-center">
                <div class="card bg-base-100 shadow-xl max-w-lg w-full">
                    <div class="card-body">
                        <h2 class="card-title mb-6">"Import Task"</h2>
                        {move || match all_data.get().flatten() {
                            Some(all) => {
                                Either::Left(
                                    view! {
                                        <form
                                            class="flex flex-col gap-6"
                                            on:submit=move |ev| {
                                                ev.prevent_default();
                                                do_import()
                                            }
                                        >
                                            <div class="form-control w-full">
                                                <label class="label">
                                                    <span class="label-text">"Select Contest"</span>
                                                </label>
                                                <select
                                                    class="select select-bordered w-full"
                                                    on:change=move |ev| {
                                                        contest_id_signal.set(event_target_value(&ev));
                                                    }
                                                >
                                                    <option
                                                        disabled
                                                        selected=move || contest_id_signal.get().is_empty()
                                                    >
                                                        "Choose a contest"
                                                    </option>
                                                    <For
                                                        each=move || all.contests.clone()
                                                        key=|c| c.contest.id
                                                        let:c
                                                    >
                                                        <option value=c.contest.id>{c.contest.name}</option>
                                                    </For>
                                                </select>
                                            </div>

                                            <div class="form-control">
                                                <label class="label cursor-pointer justify-start gap-4">
                                                    <input
                                                        type="checkbox"
                                                        class="checkbox checkbox-primary"
                                                        checked=update_signal
                                                        on:change=move |ev| {
                                                            update_signal.set(event_target_checked(&ev));
                                                        }
                                                    />
                                                    <span class="label-text">"Update existing tasks"</span>
                                                </label>
                                            </div>

                                            <div class="form-control w-full">
                                                <label class="label">
                                                    <span class="label-text">
                                                        "Task ZIP files ("
                                                        <code class="text-primary">
                                                            "task-maker-tools export-booklet"
                                                        </code> ")"
                                                    </span>
                                                </label>
                                                <input
                                                    type="file"
                                                    class="file-input file-input-bordered w-full"
                                                    accept="application/zip"
                                                    multiple=true
                                                    node_ref=input_ref
                                                />
                                            </div>

                                            <div class="card-actions justify-end mt-4">
                                                <button
                                                    type="submit"
                                                    class="btn btn-primary w-full"
                                                    disabled=loading
                                                >
                                                    {move || {
                                                        if loading.get() {
                                                            view! {
                                                                <span class="loading loading-spinner loading-xs"></span>
                                                                "Importing..."
                                                            }
                                                                .into_any()
                                                        } else {
                                                            view! { "Import" }.into_any()
                                                        }
                                                    }}
                                                </button>
                                            </div>
                                        </form>
                                    },
                                )
                            }
                            None => {
                                Either::Right(
                                    view! {
                                        <div class="flex flex-col items-center py-8">
                                            <span class="loading loading-spinner loading-lg text-primary"></span>
                                            <p class="mt-4 text-base-content/60">
                                                "Loading contests..."
                                            </p>
                                        </div>
                                    },
                                )
                            }
                        }}
                    </div>
                </div>

                <div class="card bg-base-100 shadow-xl max-w-lg w-full">
                    <div class="card-body">
                        <h2 class="card-title mb-6">"Create Contest Day"</h2>
                        <form
                            class="flex flex-col gap-6"
                            on:submit=move |ev| {
                                ev.prevent_default();
                                do_create_contest()
                            }
                        >
                            <div class="form-control w-full">
                                <label class="label">
                                    <span class="label-text">"Contest Name"</span>
                                </label>
                                <input
                                    type="text"
                                    class="input input-bordered w-full"
                                    placeholder="e.g. Day 1"
                                    prop:value=new_contest_name
                                    on:input=move |ev| {
                                        new_contest_name.set(event_target_value(&ev));
                                    }
                                />
                            </div>
                            <div class="card-actions justify-end mt-4">
                                <button
                                    type="submit"
                                    class="btn btn-primary w-full"
                                    disabled=contest_loading
                                >
                                    {move || {
                                        if contest_loading.get() {
                                            view! {
                                                <span class="loading loading-spinner loading-xs"></span>
                                                "Creating..."
                                            }
                                                .into_any()
                                        } else {
                                            view! { "Create" }.into_any()
                                        }
                                    }}
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            </div>

            <div class="w-full">
                <h2 class="text-2xl font-bold mb-4">"Tasks"</h2>
                <div class="flex flex-col gap-8">
                    {move || match all_data.get().flatten() {
                        Some(all) => {
                            Either::Left(
                                view! {
                                    <For
                                        each=move || all.contests.clone()
                                        key=|c| c.contest.id
                                        let:c
                                    >
                                        <div class="card bg-base-100 shadow-md">
                                            <div class="card-body p-4">
                                                <div class="flex justify-between items-center border-b pb-2 mb-4">
                                                    <h3 class="card-title text-lg">
                                                        "Contest: " {c.contest.name}
                                                    </h3>
                                                    <a
                                                        class="btn btn-outline btn-sm"
                                                        href=format!(
                                                            "/api/admin/export/translations/{}",
                                                            c.contest.id,
                                                        )
                                                        target="_blank"
                                                    >
                                                        "Export Translations"
                                                    </a>
                                                </div>
                                                <div class="overflow-x-auto">
                                                    <table class="table table-zebra w-full">
                                                        <thead>
                                                            <tr>
                                                                <th>"Task Name"</th>
                                                                <th class="w-32 whitespace-nowrap">"Actions"</th>
                                                            </tr>
                                                        </thead>
                                                        <tbody>
                                                            <For each=move || c.tasks.clone() key=|t| t.id let:t>
                                                                <tr>
                                                                    <td>{t.name}</td>
                                                                    <td class="whitespace-nowrap">
                                                                        <A
                                                                            href=format!("/admin/task/{}/edit", t.id)
                                                                            attr:class="btn btn-secondary btn-sm"
                                                                        >
                                                                            "Edit Files"
                                                                        </A>
                                                                    </td>
                                                                </tr>
                                                            </For>
                                                        </tbody>
                                                    </table>
                                                </div>
                                            </div>
                                        </div>
                                    </For>
                                },
                            )
                        }
                        None => Either::Right(view! { <p>"Loading tasks..."</p> }),
                    }}
                </div>
            </div>
        </div>
    }
}
