use common::language::{Language, ToggleLanguagePublicStatusPayload};
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;

use crate::api_wrapper::api_post;
use crate::{show_error, show_success};

#[component]
pub fn LanguagesTable(transl_langs: Vec<Language>) -> impl IntoView {
    view! {
        <div class="card bg-base-100 shadow-none border border-base-300">
            <div class="card-body">
                <h2 class="card-title mb-4">"Languages"</h2>
                <div class="overflow-x-auto">
                    <table class="table table-zebra w-full">
                        <thead>
                            <tr>
                                <th>"Code"</th>
                                <th>"Public"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For each=move || transl_langs.clone() key=|lang| lang.id let(lang)>
                                <tr>
                                    <td>{lang.code.clone()}</td>
                                    <td>
                                        <LanguagePublicCheckbox
                                            language_id=lang.id
                                            public=lang.public
                                        />
                                    </td>
                                </tr>
                            </For>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}

#[component]
fn LanguagePublicCheckbox(language_id: i64, public: bool) -> impl IntoView {
    let (checked, set_checked) = signal(public);
    let (original, set_original) = signal(public);

    Effect::new(move |_| {
        let val = checked.get();
        let orig = original.get_untracked();
        if val == orig {
            return;
        }

        spawn_local_scoped(async move {
            let payload = ToggleLanguagePublicStatusPayload {
                language_id,
                public: val,
            };
            match api_post("/api/user/toggle_language_public_status", &payload).await {
                Ok(()) => {
                    show_success!("Language public status updated successfully");
                    set_original.set(val);
                }
                Err(e) => {
                    show_error!("Failed to update language public status: {e}");
                    set_checked.set(orig);
                }
            }
        });
    });

    view! {
        <input
            type="checkbox"
            class="checkbox checkbox-primary"
            checked=checked
            on:change=move |ev| {
                set_checked.set(event_target_checked(&ev));
            }
        />
    }
}
