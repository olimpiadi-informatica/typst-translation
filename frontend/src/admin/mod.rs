use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use thaw::Button;

use crate::header::Header;

pub mod import_task;

#[component]
pub fn AdminHomePage() -> impl IntoView {
    let navigate = use_navigate();
    view! {
        <Header title="Admin Panel" />
        <Button on_click=move |_| navigate(
            "/admin/import_task",
            Default::default(),
        )>"Import Task"</Button>
    }
}
