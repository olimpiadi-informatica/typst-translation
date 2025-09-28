use common::contest::Contest;
use common::translation::ImportTaskPayload;
use js_sys::Uint8Array;
use leptos::either::Either;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use thaw::{Button, ButtonAppearance, ButtonType, Checkbox, Flex, Select, Spinner};
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
            let update = update.get_untracked();
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
                    update,
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
        <Header title="Import Task" />
        {move || match contests.get().flatten() {
            Some(contests) => {
                Either::Left(
                    view! {
                        <form on:submit={
                            let do_import = do_import.clone();
                            move |ev| {
                                ev.prevent_default();
                                do_import()
                            }
                        }>
                            <Flex vertical=true style="max-width: 400px; margin: auto">
                                <Select value=contest>
                                    <For
                                        each=move || contests.clone()
                                        key=|contest| contest.id
                                        let:contest
                                    >
                                        <option value=contest.id>{contest.name}</option>
                                    </For>
                                </Select>
                                <Checkbox checked=update label="Update" />
                                <input
                                    type="file"
                                    accept="application/zip"
                                    multiple=true
                                    node_ref=input_ref
                                />
                                <Button
                                    button_type=ButtonType::Submit
                                    appearance=ButtonAppearance::Primary
                                    loading
                                >
                                    "Import"
                                </Button>
                            </Flex>
                        </form>
                    },
                )
            }
            None => Either::Right(view! { <Spinner label="Loading..." /> }),
        }}
    }
}
