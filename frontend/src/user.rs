use common::user::{ExtUser, User, WhoAmIResponse};
use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::components::Outlet;
use thaw::Spinner;

use crate::api_wrapper::api_post;
use crate::login::{AdminLoginPage, UserLoginPage};
use crate::show_error;

#[derive(Clone)]
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
        self.get_ext_user_untracked().into_user().unwrap()
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
        None => Either::Right(view! { <Spinner label="Authenticating..." /> }),
    }
}

#[component]
pub fn AdminProvider() -> impl IntoView {
    let user_provider = use_context::<ExtUserContext>().unwrap();

    move || match user_provider.resource.get() {
        Some(Some(ExtUser::Admin)) => Either::Left(view! { <Outlet /> }),
        _ => Either::Right(view! { <AdminLoginPage /> }),
    }
}

#[component]
pub fn UserProvider() -> impl IntoView {
    let user_provider = use_context::<ExtUserContext>().unwrap();

    move || match user_provider.resource.get() {
        Some(Some(ExtUser::User(_))) => Either::Left(view! { <Outlet /> }),
        _ => Either::Right(view! { <UserLoginPage /> }),
    }
}
