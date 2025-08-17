use frontend::{App, init_logging};

fn main() {
    init_logging();
    leptos::mount::mount_to_body(move || leptos::view! { <App /> });
}
