use leptos::prelude::*;
use leptos_router::components::{ParentRoute, Route, Router, Routes};
use leptos_router::path;
use leptos_use::{ColorMode, UseColorModeOptions, use_color_mode_with_options};

use crate::admin::AdminHomePage;
use crate::admin::import_task::ImportTaskPage;
use crate::compare::Compare;
use crate::compilation_manager::CompilationManager;
use crate::edit::EditPage;
use crate::home::HomePage;
use crate::staff::StaffHomePage;
use crate::staff::printing::PrintingPage;
use crate::toast::ToastProvider;
use crate::user::{AdminProvider, ExtUserProvider, StaffProvider, UserProvider};

pub fn wrap_with_current_owner(cl: impl Fn() + Clone) -> impl Fn() + Clone {
    let owner = Owner::current().unwrap();
    move || owner.with(cl.clone())
}

#[component]
pub fn App() -> impl IntoView {
    let color_mode = use_color_mode_with_options(
        UseColorModeOptions::default()
            .cookie_enabled(true)
            .cookie_name("typst-translation-color-mode"),
    );

    provide_context(CompilationManager::new());

    Effect::new(move || {
        let mode = color_mode.mode.get();
        if let Some(document) = web_sys::window().and_then(|w| w.document())
            && let Some(root) = document.document_element()
        {
            let theme = match mode {
                ColorMode::Dark => "forest",
                ColorMode::Light => "nord",
                _ => "forest",
            };
            let _ = root.set_attribute("data-theme", theme);
        }
    });

    view! {
        <ToastProvider>
            <ExtUserProvider>
                <Router>
                    <Routes fallback=|| view! { <h1>"404 Not found."</h1> }>
                        <ParentRoute path=path!("/admin") view=AdminProvider>
                            <Route path=path!("") view=AdminHomePage />
                            <Route path=path!("import_task") view=ImportTaskPage />
                        </ParentRoute>
                        <ParentRoute path=path!("/staff") view=StaffProvider>
                            <Route path=path!("") view=StaffHomePage />
                            <Route path=path!("printing") view=PrintingPage />
                        </ParentRoute>
                        <ParentRoute path=path!("") view=UserProvider>
                            <Route path=path!("/") view=HomePage />
                            <Route path=path!("/task/:task") view=EditPage />
                            <Route path=path!("/task/:task/lang/:lang") view=EditPage />
                            <Route path=path!("/compare/:task_name/:id_old/:id_new") view=Compare />
                        </ParentRoute>
                    </Routes>
                </Router>
            </ExtUserProvider>
        </ToastProvider>
    }
}
