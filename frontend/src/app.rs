use leptos::prelude::*;
use leptos_router::components::{ParentRoute, Route, Router, Routes};
use leptos_router::path;
use leptos_use::{ColorMode, UseColorModeOptions, use_color_mode_with_options};
use thaw::{ConfigProvider, Theme, ToasterProvider};

use crate::admin::AdminHomePage;
use crate::admin::import_task::ImportTaskPage;
use crate::compilation_manager::CompilationManager;
use crate::edit::EditPage;
use crate::home::HomePage;
use crate::user::{AdminProvider, ExtUserProvider, UserProvider};

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

    provide_context(CompilationManager::new());

    Effect::new(move || {
        theme.set(theme_from_color_mode(color_mode.mode.get()));
    });

    view! {
        <ConfigProvider theme>
            <ToasterProvider>
                <ExtUserProvider>
                    <Router>
                        <Routes fallback=|| view! { <h1>"404 Not found."</h1> }>
                            <ParentRoute path=path!("/admin") view=AdminProvider>
                                <Route path=path!("") view=AdminHomePage />
                                <Route path=path!("import_task") view=ImportTaskPage />
                            </ParentRoute>
                            <ParentRoute path=path!("") view=UserProvider>
                                <Route path=path!("/") view=HomePage />
                                <Route path=path!("/task/:task") view=EditPage />
                                <Route path=path!("/task/:task/lang/:lang") view=EditPage />
                            </ParentRoute>
                        </Routes>
                    </Router>
                </ExtUserProvider>
            </ToasterProvider>
        </ConfigProvider>
    }
}
