use leptos::prelude::*;
use leptos_router::components::A;
use thaw::Button;

use crate::header::Header;

pub mod import_task;

#[component]
pub fn AdminHomePage() -> impl IntoView {
    view! {
        <Header title="Admin Panel" />
        <A href="/admin/import_task">
            <Button>"Import Task"</Button>
        </A>
    }
}
