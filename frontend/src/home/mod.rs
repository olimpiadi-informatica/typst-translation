use common::contestant::Contestant;
use common::language::{AssignLanguagePayload, Language, ToggleLanguagePublicStatusPayload};
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use thaw::{
    Checkbox, Select, Spinner, Table, TableBody, TableCell, TableCellLayout, TableHeader,
    TableHeaderCell, TableRow,
};

use crate::api_wrapper::api_post;
use crate::user::UserContext;
use crate::{show_error, show_success};

#[component]
pub fn HomePage() -> impl IntoView {
    let available_languages = LocalResource::<Vec<Language>>::new(|| async {
        match api_post("/api/user/available_languages", &()).await {
            Ok(names) => names,
            Err(e) => {
                show_error!("Failed to fetch available languages: {e}");
                Vec::new()
            }
        }
    });

    let contestants = LocalResource::<Vec<Contestant>>::new(|| async {
        match api_post("/api/user/contestants_with_languages", &()).await {
            Ok(names) => names,
            Err(e) => {
                show_error!("Failed to fetch contestant list: {e}");
                Vec::new()
            }
        }
    });

    let user_context = expect_context::<UserContext>();
    let user = user_context.get_user_untracked();
    let user_id = user.as_user().unwrap().id;

    move || {
        match (contestants.get(), available_languages.get()) {
        (Some(contestants), Some(languages)) => {
            let my_langs = languages
                .iter()
                .filter(|lang| lang.user_id == user_id)
                .cloned()
                .collect::<Vec<_>>();

            view! {
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHeaderCell resizable=true>"Code"</TableHeaderCell>
                            <TableHeaderCell resizable=true>"Name"</TableHeaderCell>
                            <TableHeaderCell resizable=true>"Online"</TableHeaderCell>
                            <TableHeaderCell resizable=true>"Language"</TableHeaderCell>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {contestants
                            .into_iter()
                            .map(move |contestant| {
                                let languages = languages.clone();
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
                                                    langs=languages
                                                    selected=contestant.language_id
                                                    contestant_id=contestant.id
                                                    user_id=user_id
                                                />
                                            </TableCellLayout>
                                        </TableCell>
                                    </TableRow>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </TableBody>
                </Table>

                // skip 3em
                <div style="height: 3em;" />

                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHeaderCell resizable=true>"Code"</TableHeaderCell>
                            <TableHeaderCell resizable=true>"Public"</TableHeaderCell>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {my_langs
                            .into_iter()
                            .map(|lang| {
                                view! {
                                    <TableRow>
                                        <TableCell>
                                            <TableCellLayout>{lang.code.clone()}</TableCellLayout>
                                        </TableCell>
                                        <TableCell>
                                            <TableCellLayout>
                                                <LanguagePublicCheckbox
                                                    language_id=lang.id
                                                    public=lang.public
                                                />
                                            </TableCellLayout>
                                        </TableCell>
                                    </TableRow>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </TableBody>
                </Table>
            }
        }
        .into_any(),
        _ => view! { <Spinner /> }.into_any(),
    }
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

#[component]
fn LanguagePublicCheckbox(language_id: i64, public: bool) -> impl IntoView {
    let checked = RwSignal::new(public);
    let original = RwSignal::new(public);

    Effect::new(move |_| {
        spawn_local_scoped(async move {
            let val = checked.get();
            let orig = original.get_untracked();
            if val == orig {
                return;
            }
            let payload = ToggleLanguagePublicStatusPayload {
                language_id,
                public: val,
            };
            match api_post("/api/user/toggle_language_public_status", &payload).await {
                Ok(()) => {
                    show_success!("Language public status updated successfully");
                    original.set(val);
                }
                Err(e) => {
                    show_error!("Failed to update language public status: {e}");
                    checked.set(orig);
                }
            }
        });
    });

    view! { <Checkbox checked /> }
}
