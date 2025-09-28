use leptos::prelude::*;
use leptos_router::components::A;
use thaw::{Button, ButtonAppearance};

use crate::header::Header;

pub mod import_task;

#[component]
pub fn AdminHomePage() -> impl IntoView {
    view! {
        <Header title="Admin Panel" />
        <A href="/admin/import_task">
            <Button appearance=ButtonAppearance::Subtle>"Import Task"</Button>
        </A>
    }
}
