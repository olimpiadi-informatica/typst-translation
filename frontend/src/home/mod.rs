use common::contest::ContestWithTasksAndStatus;
use common::contestant::Contestant;
use common::language::Language;
use leptos::either::Either;
use leptos::prelude::*;
use thaw::{Flex, FlexGap, Spinner};

use crate::api_wrapper::{api_get, api_post};
use crate::header::Header;
use crate::home::contestants::ContestantsTable;
use crate::home::languages::LanguagesTable;
use crate::home::translations::Translations;
use crate::show_error;
use crate::user::UserContext;

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

    let user_context = expect_context::<UserContext>();
    let user = user_context.get_user_untracked();
    let user_id = user.as_user().unwrap().id;

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
                <Flex vertical=true gap=FlexGap::Large style="max-width: 1200px; margin: auto">
                    <Header title="Translation System".to_owned() />
                    <Flex gap=FlexGap::Large>
                        <ContestantsTable contestants avail_langs />
                        <LanguagesTable transl_langs />
                    </Flex>
                    <Translations contests all_langs />
                </Flex>
            })
        }
        _ => Either::Right(view! { <Spinner label="Loading..." /> }),
    }
}
