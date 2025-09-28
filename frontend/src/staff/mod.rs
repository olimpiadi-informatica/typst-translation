use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use thaw::{Button, ButtonGroup};

use crate::header::Header;

pub mod printing;

#[component]
pub fn StaffHomePage() -> impl IntoView {
    view! {
        <Header title="Staff Panel" />
        <ButtonGroup>
            <Button on_click=move |_| {
                let navigate = use_navigate();
                navigate("/staff/printing", Default::default())
            }>"Printing"</Button>
        </ButtonGroup>
    }
}
