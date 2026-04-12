use common::contest::ContestWithTasksAndStatus;
use common::contestant::Contestant;
use common::language::Language;
use leptos::either::Either;
use leptos::prelude::*;

use crate::api_wrapper::{api_get, api_post};
use crate::header::Header;
use crate::home::contestants::ContestantsTable;
use crate::home::languages::LanguagesTable;
use crate::home::translations::Translations;
use crate::show_error;
use crate::user::ExtUserContext;

mod contestants;
mod languages;
mod translations;

#[component]
pub fn HomePage() -> impl IntoView {
    let contestants = LocalResource::<Option<Vec<Contestant>>>::new(|| async {
        match api_post("/api/user/contestants_with_languages", &()).await {
            Ok(contestants) => Some(contestants),
            Err(e) => {
                show_error!("Failed to fetch contestants: {}", e);
                None
            }
        }
    });

    let all_langs = LocalResource::<Option<Vec<Language>>>::new(|| async {
        match api_get("/api/user/all_languages").await {
            Ok(langs) => Some(langs),
            Err(e) => {
                show_error!("Failed to fetch all languages: {}", e);
                None
            }
        }
    });

    let contests = LocalResource::<Option<Vec<ContestWithTasksAndStatus>>>::new(|| async {
        match api_get("/api/user/translation_status").await {
            Ok(data) => Some(data),
            Err(e) => {
                show_error!("Failed to fetch translation status: {}", e);
                None
            }
        }
    });

    let user_context = expect_context::<ExtUserContext>();
    let user = user_context.get_user_untracked();
    let user_id = user.id;

    move || match (
        contestants.get().flatten(),
        all_langs.get().flatten(),
        contests.get().flatten(),
    ) {
        (Some(contestants), Some(all_langs), Some(contests)) => {
            let avail_langs = all_langs
                .iter()
                .filter(|lang| lang.public || lang.user_id == user_id)
                .cloned()
                .collect::<Vec<_>>();
            let transl_langs = all_langs
                .iter()
                .filter(|lang| lang.user_id == user_id)
                .cloned()
                .collect::<Vec<_>>();
            Either::Left(view! {
                <div class="container mx-auto max-w-7xl p-4 flex flex-col gap-8">
                    <Header noback=true title=Signal::derive(|| "Translation System".to_string()) />
                    <div class="flex flex-col md:flex-row gap-8">
                        <div class="flex-1">
                            <ContestantsTable contestants avail_langs />
                        </div>
                        <div class="flex-1">
                            <LanguagesTable transl_langs />
                        </div>
                    </div>
                    <Translations contests all_langs />
                </div>
            })
        }
        _ => Either::Right(view! {
            <div class="flex justify-center items-center h-screen">
                <span class="loading loading-spinner loading-lg"></span>
                <span class="ml-2">"Loading..."</span>
            </div>
        }),
    }
}
