use common::contest::Contest;
use common::translation::ImportTaskPayload;
use js_sys::Uint8Array;
use leptos::either::Either;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use crate::api_wrapper::{api_get, api_post};
use crate::app::wrap_with_current_owner;
use crate::header::Header;
use crate::{show_error, show_success};

#[component]
pub fn ImportTaskPage() -> impl IntoView {
    let contests = LocalResource::<Option<Vec<Contest>>>::new(move || async move {
        match api_get("/api/contests/get_all").await {
            Ok(contests) => Some(contests),
            Err(e) => {
                show_error!("Failed to fetch contests: {e}");
                None
            }
        }
    });

    let input_ref: NodeRef<html::Input> = NodeRef::new();
    let contest = RwSignal::new(String::new());
    let update = RwSignal::new(false);
    let loading = RwSignal::new(false);

    let do_import = wrap_with_current_owner(move || {
        spawn_local_scoped(async move {
            loading.set(true);

            let contest_id = contest
                .get_untracked()
                .parse::<i64>()
                .expect("Invalid contest ID");
            let update_val = update.get_untracked();
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

                tracing::info!("Selected file: {}, size: {}", file_name, zip_file.len());

                let payload = ImportTaskPayload {
                    contest_id,
                    update: update_val,
                    zip_file,
                };
                match api_post("/api/tasks/import", &payload).await {
                    Ok(()) => {
                        show_success!("Imported file: {}", file_name);
                    }
                    Err(e) => {
                        show_error!("Failed to import file {}: {}", file_name, e);
                    }
                }
            }

            loading.set(false);
        });
    });

    view! {
        <Header title=Signal::derive(|| "Import Task".to_string()) />
        <div class="container mx-auto max-w-lg p-8">
            <div class="card bg-base-100 shadow-xl">
                <div class="card-body">
                    <h2 class="card-title mb-6">"Import Task"</h2>
                    {move || match contests.get().flatten() {
                        Some(contests) => {
                            Either::Left(
                                view! {
                                    <form
                                        class="flex flex-col gap-6"
                                        on:submit={
                                            let do_import = do_import.clone();
                                            move |ev| {
                                                ev.prevent_default();
                                                do_import()
                                            }
                                        }
                                    >
                                        <div class="form-control w-full">
                                            <label class="label">
                                                <span class="label-text">"Select Contest"</span>
                                            </label>
                                            <select
                                                class="select select-bordered w-full"
                                                on:change=move |ev| {
                                                    contest.set(event_target_value(&ev));
                                                }
                                            >
                                                <option disabled selected=move || contest.get().is_empty()>
                                                    "Choose a contest"
                                                </option>
                                                <For
                                                    each=move || contests.clone()
                                                    key=|contest| contest.id
                                                    let:contest
                                                >
                                                    <option value=contest.id>{contest.name}</option>
                                                </For>
                                            </select>
                                        </div>

                                        <div class="form-control">
                                            <label class="label cursor-pointer justify-start gap-4">
                                                <input
                                                    type="checkbox"
                                                    class="checkbox checkbox-primary"
                                                    checked=update
                                                    on:change=move |ev| {
                                                        update.set(event_target_checked(&ev));
                                                    }
                                                />
                                                <span class="label-text">"Update existing tasks"</span>
                                            </label>
                                        </div>

                                        <div class="form-control w-full">
                                            <label class="label">
                                                <span class="label-text">"Task ZIP files"</span>
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
        </div>
    }
}
