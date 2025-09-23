use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div>"Welcome to the Home Page"</div>
        <a href="/edit">"Go to Editor"</a>
    }
}
