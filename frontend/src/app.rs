use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use leptos_use::{ColorMode, UseColorModeOptions, use_color_mode_with_options};
use thaw::{ConfigProvider, Theme, ToasterProvider};

use crate::edit::EditPage;
use crate::home::HomePage;
use crate::user::UserProvider;

pub fn wrap_with_current_owner(cl: impl Fn() + Clone) -> impl Fn() + Clone {
    let owner = Owner::current().unwrap();
    move || owner.with(cl.clone())
}

fn theme_from_color_mode(color_mode: ColorMode) -> Theme {
    if color_mode == ColorMode::Dark {
        Theme::dark()
    } else {
        Theme::light()
    }
}

#[component]
pub fn App() -> impl IntoView {
    let color_mode = use_color_mode_with_options(
        UseColorModeOptions::default()
            .cookie_enabled(true)
            .cookie_name("typst-translation-color-mode"),
    );

    let theme = RwSignal::new(Theme::dark());

    Effect::new(move || {
        theme.set(theme_from_color_mode(color_mode.mode.get()));
    });

    view! {
        <ConfigProvider theme>
            <ToasterProvider>
                <UserProvider>
                    <Router>
                        <Routes fallback=|| "Not found.">
                            <Route path=path!("/") view=HomePage />
                            <Route path=path!("/edit") view=EditPage />
                        </Routes>
                    </Router>
                </UserProvider>
            </ToasterProvider>
        </ConfigProvider>
    }
}
