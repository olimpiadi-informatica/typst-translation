use common::user::WhoAmIResponse;
use leptos::prelude::*;

use crate::api_wrapper::api_post;
use crate::login::LoginPage;
use crate::show_error;

#[derive(Clone)]
pub struct UserContext {
    resource: LocalResource<WhoAmIResponse>,
}

impl UserContext {
    pub fn get_user(&self) -> WhoAmIResponse {
        self.resource
            .get()
            .clone()
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
                WhoAmIResponse::Nobody
            }
        }
    });

    provide_context(UserContext { resource: user });

    let children = children.into_inner();
    move || match user.get() {
        Some(WhoAmIResponse::Nobody) => view! { <LoginPage /> }.into_any(),
        Some(_) => view! { {children()} }.into_any(),
        None => view! { "Loading" }.into_any(),
    }
}
