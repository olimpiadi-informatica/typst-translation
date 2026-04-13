use leptos::prelude::*;

pub mod import_task;

#[component]
pub fn AdminHomePage() -> impl IntoView {
    view! {
        <div class="p-4">
            <h1 class="text-2xl font-bold">"Welcome to the Admin Panel"</h1>
            <p class="mt-2">"Select an option from the tabs above."</p>
        </div>
    }
}
