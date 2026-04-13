use common::user::LoginParams;
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;

use crate::api_wrapper::api_post;
use crate::user::ExtUserContext;
use crate::util::Card;
use crate::{show_error, show_success};

#[component]
pub fn PasswordLoginPage(
    title: &'static str,
    endpoint: &'static str,
    theme: &'static str,
) -> impl IntoView {
    let password = RwSignal::new("".to_string());

    let do_login = move || {
        spawn_local_scoped(async move {
            let params = password.get_untracked();

            match api_post(endpoint, &params).await {
                Ok(()) => {}
                Err(e) => {
                    show_error!("Login failed: {e}");
                    return;
                }
            }

            show_success!("Login successful");
            let user_context = expect_context::<ExtUserContext>();
            user_context.refetch();
        });
    };

    view! {
        <div class="flex items-center justify-center h-screen bg-base-200">
            <Card title=title class=format!("w-96 !shadow-xl border-t-4 border-{}", theme)>
                <form on:submit=move |ev| {
                    ev.prevent_default();
                    do_login()
                }>
                    <div class="form-control w-full">
                        <label class="label">
                            <span class="label-text">"Password"</span>
                        </label>
                        <input
                            type="password"
                            placeholder=format!("{} Password", title)
                            class="input input-bordered w-full"
                            autocomplete="current-password"
                            on:input=move |ev| password.set(event_target_value(&ev))
                            prop:value=password
                            required
                        />
                    </div>
                    <div class="card-actions justify-end mt-6">
                        <button type="submit" class=format!("btn btn-{} w-full", theme)>
                            "Login"
                        </button>
                    </div>
                </form>
            </Card>
        </div>
    }
}

#[component]
pub fn UserLoginPage() -> impl IntoView {
    let username = RwSignal::new("".to_string());
    let password = RwSignal::new("".to_string());

    let do_login = move || {
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

            show_success!("Login successful");
            let user_context = expect_context::<ExtUserContext>();
            user_context.refetch();
        });
    };

    view! {
        <div class="flex items-center justify-center h-screen bg-base-200">
            <Card title="Login" class="w-96 !shadow-xl">
                <form on:submit=move |ev| {
                    ev.prevent_default();
                    do_login()
                }>
                    <div class="form-control w-full">
                        <label class="label">
                            <span class="label-text">"Username"</span>
                        </label>
                        <input
                            type="text"
                            placeholder="Username"
                            class="input input-bordered w-full"
                            autocomplete="username"
                            on:input=move |ev| username.set(event_target_value(&ev))
                            prop:value=username
                            required
                        />
                    </div>
                    <div class="form-control w-full mt-4">
                        <label class="label">
                            <span class="label-text">"Password"</span>
                        </label>
                        <input
                            type="password"
                            placeholder="Password"
                            class="input input-bordered w-full"
                            autocomplete="current-password"
                            on:input=move |ev| password.set(event_target_value(&ev))
                            prop:value=password
                            required
                        />
                    </div>
                    <div class="card-actions justify-end mt-6">
                        <button type="submit" class="btn btn-primary w-full">
                            "Login"
                        </button>
                    </div>
                </form>
            </Card>
        </div>
    }
}

#[component]
pub fn AdminLoginPage() -> impl IntoView {
    view! { <PasswordLoginPage title="Admin" endpoint="/api/admin/login" theme="error" /> }
}

#[component]
pub fn StaffLoginPage() -> impl IntoView {
    view! { <PasswordLoginPage title="Staff" endpoint="/api/staff/login" theme="secondary" /> }
}
