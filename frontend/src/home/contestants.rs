use common::contestant::Contestant;
use common::language::{AssignLanguagePayload, Language};
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use thaw::{
    Select, Table, TableBody, TableCell, TableCellLayout, TableHeader, TableHeaderCell, TableRow,
};

use crate::api_wrapper::api_post;
use crate::{show_error, show_success};

#[component]
pub fn ContestantsTable(contestants: Vec<Contestant>, avail_langs: Vec<Language>) -> impl IntoView {
    let rows = contestants
        .into_iter()
        .map(move |contestant| {
            let avail_langs = avail_langs.clone();
            view! {
                <TableRow>
                    <TableCell>
                        <TableCellLayout>{contestant.code.clone()}</TableCellLayout>
                    </TableCell>
                    <TableCell>
                        <TableCellLayout>{contestant.name.clone()}</TableCellLayout>
                    </TableCell>
                    <TableCell>
                        <TableCellLayout>
                            {if contestant.online_bit { "Yes" } else { "No" }}
                        </TableCellLayout>
                    </TableCell>
                    <TableCell>
                        <TableCellLayout>
                            <ContestantLangSelect
                                langs=avail_langs
                                selected=contestant.language_id
                                contestant_id=contestant.id
                                user_id=contestant.user_id
                            />
                        </TableCellLayout>
                    </TableCell>
                </TableRow>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <div>
            <h1>"Contestants"</h1>
            <Table>
                <TableHeader>
                    <TableRow>
                        <TableHeaderCell resizable=true>"Code"</TableHeaderCell>
                        <TableHeaderCell resizable=true>"Name"</TableHeaderCell>
                        <TableHeaderCell resizable=true>"Online"</TableHeaderCell>
                        <TableHeaderCell resizable=true>"Language"</TableHeaderCell>
                    </TableRow>
                </TableHeader>
                <TableBody>{rows}</TableBody>
            </Table>
        </div>
    }
}

#[component]
fn ContestantLangSelect(
    langs: Vec<Language>,
    selected: Option<i64>,
    contestant_id: i64,
    user_id: i64,
) -> impl IntoView {
    let value = RwSignal::new(selected.map(|id| id.to_string()).unwrap_or_default());
    let original = RwSignal::new(selected);

    Effect::new(move |_| {
        spawn_local_scoped(async move {
            let val = value.get().parse::<i64>().ok();
            let orig = original.get_untracked();
            if val == orig {
                return;
            }
            let payload = AssignLanguagePayload {
                contestant_id,
                language_id: val,
            };
            match api_post("/api/user/assign_language_to_contestant", &payload).await {
                Ok(()) => {
                    show_success!("Contestant language updated successfully");
                    original.set(val);
                }
                Err(e) => {
                    show_error!("Failed to update contestant language: {e}");
                    value.set(val.map(|id| id.to_string()).unwrap_or_default());
                }
            }
        });
    });

    view! {
        <Select value=value>
            <option />
            <optgroup label="Your Languages">
                {langs
                    .iter()
                    .filter(|lang| lang.user_id == user_id)
                    .map(|lang| {
                        view! {
                            <option value=lang.id.to_string() selected=Some(lang.id) == selected>
                                {lang.code.clone()}
                            </option>
                        }
                    })
                    .collect::<Vec<_>>()}
            </optgroup>
            <optgroup label="Public Languages">
                {langs
                    .iter()
                    .filter(|lang| lang.user_id != user_id)
                    .map(|lang| {
                        view! {
                            <option value=lang.id.to_string() selected=Some(lang.id) == selected>
                                {lang.code.clone()}
                            </option>
                        }
                    })
                    .collect::<Vec<_>>()}
            </optgroup>
        </Select>
    }
}
