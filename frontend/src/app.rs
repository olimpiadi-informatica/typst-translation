use leptos::prelude::*;
use leptos_router::components::{ParentRoute, Route, Router, Routes};
use leptos_router::path;
use leptos_use::{ColorMode, UseColorModeOptions, use_color_mode_with_options};

use crate::admin::edit_task::AdminEditTaskPage;
use crate::admin::tasks::AdminTasksPage;
use crate::admin::users::AdminUsersPage;
use crate::compare::Compare;
use crate::compilation_manager::CompilationManager;
use crate::edit::EditPage;
use crate::home::HomePage;
use crate::staff::printing::PrintingPage;
use crate::toast::ToastProvider;
use crate::user::{
    AdminPanelLayout, AdminProvider, ExtUserProvider, StaffPanelLayout, StaffProvider, UserProvider,
};

#[component]
pub fn App() -> impl IntoView {
    let color_mode = use_color_mode_with_options(
        UseColorModeOptions::default()
            .cookie_enabled(true)
            .cookie_name("typst-translation-color-mode"),
    );

    provide_context(CompilationManager::new());
    provide_context(color_mode.mode);

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
                            <ParentRoute path=path!("") view=AdminPanelLayout>
                                <Route path=path!("") view=AdminUsersPage />
                                <Route path=path!("tasks") view=AdminTasksPage />
                                <Route path=path!("printing") view=PrintingPage />
                                <Route path=path!("users") view=AdminUsersPage />
                            </ParentRoute>
                            <Route path=path!("task/:task/edit") view=AdminEditTaskPage />
                        </ParentRoute>
                        <ParentRoute path=path!("/staff") view=StaffProvider>
                            <ParentRoute path=path!("") view=StaffPanelLayout>
                                <Route path=path!("") view=PrintingPage />
                                <Route path=path!("printing") view=PrintingPage />
                            </ParentRoute>
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
