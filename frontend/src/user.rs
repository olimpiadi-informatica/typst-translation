use common::user::{ExtUser, WhoAmIResponse};
use leptos::prelude::*;

use crate::api_wrapper::api_post;
use crate::login::LoginPage;
use crate::show_error;

#[derive(Clone)]
pub struct UserContext {
    resource: LocalResource<Option<ExtUser>>,
}

impl UserContext {
    pub fn get_user(&self) -> ExtUser {
        self.resource
            .get()
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
        Some(Some(_)) => children().into_any(),
        Some(None) => view! { <LoginPage /> }.into_any(),
        None => "Loading".into_any(),
    }
}
