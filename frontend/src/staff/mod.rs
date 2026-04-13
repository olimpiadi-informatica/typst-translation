use leptos::prelude::*;

pub mod printing;

#[component]
pub fn StaffHomePage() -> impl IntoView {
    view! {
        <div class="p-4">
            <h1 class="text-2xl font-bold">"Welcome to the Staff Panel"</h1>
            <p class="mt-2">"Select an option from the tabs above."</p>
        </div>
    }
}
