use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::header::Header;

pub mod printing;

#[component]
pub fn StaffHomePage() -> impl IntoView {
    view! {
        <Header title=Signal::derive(|| "Staff Panel".to_string()) />
        <div class="p-4">
            <div class="join">
                <button
                    class="btn btn-primary join-item"
                    on:click=move |_| {
                        let navigate = use_navigate();
                        navigate("/staff/printing", Default::default())
                    }
                >
                    "Printing"
                </button>
            </div>
        </div>
    }
}
