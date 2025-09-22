use common::user::LoginParams;
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use thaw::{
    Button, ButtonAppearance, ButtonType, Field, FieldOrientation, Flex, Input, InputRule,
    InputType,
};

use crate::api_wrapper::api_post;
use crate::app::wrap_with_current_owner;
use crate::{show_error, show_success};

#[component]
pub fn LoginPage() -> impl IntoView {
    let username = RwSignal::new("".to_string());
    let password = RwSignal::new("".to_string());

    let do_login = wrap_with_current_owner(move || {
        spawn_local_scoped(async move {
            let params = LoginParams {
                username: username.get_untracked(),
                password: password.get_untracked(),
            };

            match api_post("/api/login", &params).await {
                Ok(()) => {}
                Err(e) => {
                    show_error!("Login failed: {e}");
                    return;
                }
            }

            // TODO
            show_success!("Login successful");
        })
    });

    view! {
        <Flex style="height: 100vh; width: 100vw; justify-content: center; align-items: center;">
            <form on:submit=move |ev| {
                ev.prevent_default();
                do_login()
            }>
                <Flex vertical=true>
                    <h1>"Login to Translation System"</h1>
                    <Field label="Username" orientation=FieldOrientation::Horizontal>
                        <Input
                            autocomplete="username"
                            rules=vec![InputRule::required(true.into())]
                            value=username
                        />
                    </Field>
                    <Field label="Password" orientation=FieldOrientation::Horizontal>
                        <Input
                            autocomplete="current-password"
                            input_type=InputType::Password
                            rules=vec![InputRule::required(true.into())]
                            value=password
                        />
                    </Field>
                    <Button button_type=ButtonType::Submit appearance=ButtonAppearance::Primary>
                        "Login"
                    </Button>
                </Flex>
            </form>
        </Flex>
    }
}
