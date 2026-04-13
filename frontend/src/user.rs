use common::user::{ExtUser, User, WhoAmIResponse};
use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::components::Outlet;

use crate::api_wrapper::api_post;
use crate::header::Header;
use crate::login::{AdminLoginPage, StaffLoginPage, UserLoginPage};
use crate::show_error;

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
pub fn AdminProvider() -> impl IntoView {
    let user_provider = use_context::<ExtUserContext>().unwrap();
    let location = leptos_router::hooks::use_location();

    move || match user_provider.resource.get() {
        Some(Some(ExtUser { is_admin: true, .. })) => Either::Left(view! {
            <div class="flex flex-col h-screen">
                <Header
                    title=Signal::derive(|| "Admin Panel".to_string())
                    tabs=view! {
                        <div role="tablist" class="tabs tabs-boxed">
                            <a href="/admin/import_task" class="tab" class:tab-active=move || location.pathname.get() == "/admin/import_task">"Import Task"</a>
                            <a href="/admin/printing" class="tab" class:tab-active=move || location.pathname.get() == "/admin/printing">"Printing"</a>
                        </div>
                    }.into_any()
                />
                <div class="flex-grow overflow-auto p-4">
                    <Outlet />
                </div>
            </div>
        }),
        _ => Either::Right(view! { <AdminLoginPage /> }),
    }
}

#[component]
pub fn UserProvider() -> impl IntoView {
    let user_provider = use_context::<ExtUserContext>().unwrap();

    move || match user_provider.resource.get() {
        Some(Some(ExtUser { user: Some(_), .. })) => Either::Left(view! { <Outlet /> }),
        _ => Either::Right(view! { <UserLoginPage /> }),
    }
}

#[component]
pub fn StaffProvider() -> impl IntoView {
    let user_provider = use_context::<ExtUserContext>().unwrap();
    let location = leptos_router::hooks::use_location();

    move || match user_provider.resource.get() {
        Some(Some(ExtUser { is_admin: true, .. })) | Some(Some(ExtUser { is_staff: true, .. })) => {
            Either::Left(view! {
                <div class="flex flex-col h-screen">
                    <Header
                        title=Signal::derive(|| "Staff Panel".to_string())
                        tabs=view! {
                            <div role="tablist" class="tabs tabs-boxed">
                                <a href="/staff/printing" class="tab" class:tab-active=move || location.pathname.get() == "/staff/printing">"Printing"</a>
                            </div>
                        }.into_any()
                    />
                    <div class="flex-grow overflow-auto p-4">
                        <Outlet />
                    </div>
                </div>
            })
        }
        _ => Either::Right(view! { <StaffLoginPage /> }),
    }
}
