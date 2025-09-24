use common::user::{ExtUser, WhoAmIResponse};
use leptos::either::EitherOf3;
use leptos::prelude::*;
use thaw::Spinner;

use crate::api_wrapper::api_post;
use crate::login::LoginPage;
use crate::show_error;

#[derive(Clone)]
pub struct UserContext {
    resource: LocalResource<Option<ExtUser>>,
}

impl UserContext {
    pub fn get_user_untracked(&self) -> ExtUser {
        self.resource
            .get_untracked()
            .flatten()
            .expect("User resource should be loaded")
    }

    pub fn refetch(&self) {
        self.resource.refetch();
    }
}

#[component]
pub fn UserProvider<C>(children: TypedChildrenFn<C>) -> impl IntoView
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

    provide_context(UserContext { resource: user });

    let children = children.into_inner();
    move || match user.get() {
        Some(Some(_)) => EitherOf3::A(children()),
        Some(None) => EitherOf3::B(view! { <LoginPage /> }),
        None => EitherOf3::C(view! { <Spinner label="Authenticating..." /> }),
    }
}
