use common::language::{Language, ToggleLanguagePublicStatusPayload};
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use thaw::{
    Checkbox, Table, TableBody, TableCell, TableCellLayout, TableHeader, TableHeaderCell, TableRow,
};

use crate::api_wrapper::api_post;
use crate::{show_error, show_success};

#[component]
pub fn LanguagesTable(transl_langs: Vec<Language>) -> impl IntoView {
    let rows = transl_langs
        .into_iter()
        .map(|lang| {
            view! {
                <TableRow>
                    <TableCell>
                        <TableCellLayout>{lang.code.clone()}</TableCellLayout>
                    </TableCell>
                    <TableCell>
                        <TableCellLayout>
                            <LanguagePublicCheckbox language_id=lang.id public=lang.public />
                        </TableCellLayout>
                    </TableCell>
                </TableRow>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <div>
            <h1>"Languages"</h1>
            <Table>
                <TableHeader>
                    <TableRow>
                        <TableHeaderCell resizable=true>"Code"</TableHeaderCell>
                        <TableHeaderCell resizable=true>"Public"</TableHeaderCell>
                    </TableRow>
                </TableHeader>
                <TableBody>{rows}</TableBody>
            </Table>
        </div>
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
