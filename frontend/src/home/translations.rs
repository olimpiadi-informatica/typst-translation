use common::contest::ContestWithTasksAndStatus;
use common::language::Language;
use common::task::Task;
use leptos::prelude::*;

#[component]
pub fn Translations(
    contests: Vec<ContestWithTasksAndStatus>,
    all_langs: Vec<Language>,
) -> impl IntoView {
    view! {
        <h1>"Translations"</h1>
        {contests
            .into_iter()
            .map(move |contest| view! { <Contest contest all_langs=all_langs.clone() /> })
            .collect::<Vec<_>>()}
    }
}

#[component]
fn Contest(contest: ContestWithTasksAndStatus, all_langs: Vec<Language>) -> impl IntoView {
    let rows = contest
        .tasks
        .into_iter()
        .map(|task| {
            view! {
                <Task task all_langs=all_langs.clone() />
            }
        })
        .collect::<Vec<_>>();

    view! {
        <div style="margin-bottom: 2em; padding: 1em; border: 2px solid #000; border-radius: 6px;">
            <h2>"Contest: " {contest.contest.name}</h2>
            {rows}
        </div>
    }
}

#[component]
fn Task(task: Task, all_langs: Vec<Language>) -> impl IntoView {
    let langs = task
        .translations
        .into_iter()
        .map(move |transl| {
            let code = all_langs
                .iter()
                .find(|lang| lang.id == transl.language_id)
                .map(|lang| lang.code.clone())
                .unwrap_or_default();
            view! { {code} }
        })
        .collect::<Vec<_>>();

    view! {
        <div style="margin-bottom: 1em; padding: 0.5em; border: 1px solid #ccc; border-radius: 4px;">
            <p>"Task: " {task.name}</p>
            <div style="margin-left: 1em;">{langs}</div>
        </div>
    }
}
