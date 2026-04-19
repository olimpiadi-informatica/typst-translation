use common::contestant::Contestant;
use common::language::{AssignLanguagePayload, Language};
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;

use crate::api_wrapper::api_post;
use crate::util::Card;
use crate::{show_error, show_success};

#[component]
pub fn ContestantsTable(contestants: Vec<Contestant>, avail_langs: Vec<Language>) -> impl IntoView {
    view! {
        <Card title="Contestants">
            <div class="overflow-x-auto">
                <table class="table table-zebra w-full">
                    <thead>
                        <tr>
                            <th>"Code"</th>
                            <th>"Name"</th>
                            <th>"Online"</th>
                            <th>"Language"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For
                            each=move || contestants.clone()
                            key=|contestant| contestant.id
                            let(contestant)
                        >
                            {
                                let avail_langs = avail_langs.clone();
                                view! {
                                    <tr>
                                        <td>{contestant.code.clone()}</td>
                                        <td>{contestant.name.clone()}</td>
                                        <td>{if contestant.online_bit { "Yes" } else { "No" }}</td>
                                        <td>
                                            <ContestantLangSelect
                                                langs=avail_langs
                                                selected=contestant.language_id
                                                contestant_id=contestant.id
                                                user_id=contestant.user_id
                                            />
                                        </td>
                                    </tr>
                                }
                            }
                        </For>
                    </tbody>
                </table>
            </div>
        </Card>
    }
}

#[component]
fn ContestantLangSelect(
    langs: Vec<Language>,
    selected: Option<i64>,
    contestant_id: i64,
    user_id: i64,
) -> impl IntoView {
    let (value, set_value) = signal(selected.map(|id| id.to_string()).unwrap_or_default());
    let (original, set_original) = signal(selected);

    Effect::new(move |_| {
        let val_str = value.get();
        let val = if val_str.is_empty() {
            None
        } else {
            val_str.parse::<i64>().ok()
        };
        let orig = original.get_untracked();
        if val == orig {
            return;
        }

        spawn_local_scoped(async move {
            let payload = AssignLanguagePayload {
                contestant_id,
                language_id: val,
            };
            match api_post("/api/user/assign_language_to_contestant", &payload).await {
                Ok(()) => {
                    show_success!("Contestant language updated successfully");
                    set_original.set(val);
                }
                Err(e) => {
                    show_error!("Failed to update contestant language: {e}");
                    set_value.set(orig.map(|id| id.to_string()).unwrap_or_default());
                }
            }
        });
    });

    view! {
        <select
            class="select select-bordered select-xs w-full max-w-xs"
            on:change=move |ev| {
                set_value.set(event_target_value(&ev));
            }
        >
            <option value="" selected=selected.is_none()>"Original (English) - No translation requested"</option>
            <optgroup label="Your Languages">
                {langs
                    .iter()
                    .filter(|lang| lang.user_id == user_id)
                    .map(|lang| {
                        let id_str = lang.id.to_string();
                        view! {
                            <option value=id_str.clone() selected=Some(lang.id) == selected>
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
                        let id_str = lang.id.to_string();
                        view! {
                            <option value=id_str.clone() selected=Some(lang.id) == selected>
                                {lang.code.clone()}
                            </option>
                        }
                    })
                    .collect::<Vec<_>>()}
            </optgroup>
        </select>
    }
}
