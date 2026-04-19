use common::user::{ExtUser, User, WhoAmIResponse};
use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::components::Outlet;

use crate::api_wrapper::api_post;
use crate::header::Header;
use crate::login::{AdminLoginPage, StaffLoginPage, UserLoginPage};
use crate::show_error;
use crate::util::NavTabs;

#[derive(Clone, Copy)]
pub struct ExtUserContext {
    resource: LocalResource<Option<ExtUser>>,
}

impl ExtUserContext {
    pub fn get_ext_user_untracked(&self) -> ExtUser {
        self.resource
            .get_untracked()
            .flatten()
            .expect("User resource should be loaded")
    }

    pub fn get_user_untracked(&self) -> User {
        self.get_ext_user_untracked().user.unwrap_or_default()
    }

    pub fn refetch(&self) {
        self.resource.refetch();
    }
}

#[component]
pub fn ExtUserProvider<C>(children: TypedChildrenFn<C>) -> impl IntoView
where
    C: IntoView + 'static,
{
    let user = LocalResource::new(move || async {
        match api_post::<_, WhoAmIResponse>("/api/whoami", &()).await {
            Ok(user) => user,
            Err(e) => {
                show_error!("Failed to fetch user info: {e}");
                None
            }
        }
    });

    provide_context(ExtUserContext { resource: user });

    let children = children.into_inner();
    move || match user.get() {
        Some(_) => Either::Left(children()),
        None => Either::Right(view! {
            <div class="flex flex-col items-center justify-center h-screen">
                <span class="loading loading-spinner loading-lg text-primary"></span>
                <p class="mt-4 text-base-content/60">"Authenticating..."</p>
            </div>
        }),
    }
}

#[component]
pub fn ProtectedRoute<F, L, C>(
    condition: F,
    login_page: L,
    children: TypedChildrenFn<C>,
) -> impl IntoView
where
    F: Fn(&ExtUser) -> bool + 'static + Send,
    L: Fn() -> AnyView + 'static + Send,
    C: IntoView + 'static,
{
    let user_provider = use_context::<ExtUserContext>().expect("ExtUserContext should be provided");
    let children = children.into_inner();

    move || {
        let user = user_provider.resource.get();
        let children = children.clone();
        match user {
            Some(Some(ext_user)) if condition(&ext_user) => Either::Left(children().into_any()),
            Some(_) => Either::Right(login_page()),
            None => Either::Right(
                view! {
                    <div class="flex flex-col items-center justify-center h-screen">
                        <span class="loading loading-spinner loading-lg text-primary"></span>
                        <p class="mt-4 text-base-content/60">"Authenticating..."</p>
                    </div>
                }
                .into_any(),
            ),
        }
    }
}

#[component]
pub fn PanelLayout(
    #[prop(into)] title: Signal<String>,
    #[prop(optional)] tabs: Option<AnyView>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col h-screen">
            <Header title=title tabs=tabs.unwrap_or_else(|| ().into_any()) />
            <div class="flex-grow overflow-auto p-4">
                <Outlet />
            </div>
        </div>
    }
}

#[component]
pub fn AdminPanelLayout() -> impl IntoView {
    view! {
        <PanelLayout
            title=Signal::derive(|| "Admin Panel".to_string())
            tabs=view! {
                <NavTabs tabs=Signal::derive(|| {
                    vec![
                        ("Users".to_string(), "/admin/users".to_string()),
                        ("Tasks".to_string(), "/admin/tasks".to_string()),
                        ("Printing".to_string(), "/admin/printing".to_string()),
                    ]
                }) />
            }
                .into_any()
        />
    }
}

#[component]
pub fn AdminProvider() -> impl IntoView {
    view! {
        <ProtectedRoute
            condition=|u| u.is_admin
            login_page=|| view! { <AdminLoginPage /> }.into_any()
        >
            <Outlet />
        </ProtectedRoute>
    }
}

#[component]
pub fn UserProvider() -> impl IntoView {
    view! {
        <ProtectedRoute
            condition=|u| u.user.is_some()
            login_page=|| view! { <UserLoginPage /> }.into_any()
        >
            <Outlet />
        </ProtectedRoute>
    }
}

#[component]
pub fn StaffPanelLayout() -> impl IntoView {
    view! {
        <PanelLayout
            title=Signal::derive(|| "Staff Panel".to_string())
            tabs=view! {
                <NavTabs tabs=Signal::derive(|| {
                    vec![("Printing".to_string(), "/staff/printing".to_string())]
                }) />
            }
                .into_any()
        />
    }
}

#[component]
pub fn StaffProvider() -> impl IntoView {
    view! {
        <ProtectedRoute
            condition=|u| u.is_admin || u.is_staff
            login_page=|| view! { <StaffLoginPage /> }.into_any()
        >
            <Outlet />
        </ProtectedRoute>
    }
}
