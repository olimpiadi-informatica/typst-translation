use common::user::LoginParams;
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;

use crate::api_wrapper::api_post;
use crate::user::ExtUserContext;
use crate::{show_error, show_success};

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
            <div class="card w-96 bg-base-100 shadow-xl">
                <div class="card-body">
                    <h1 class="card-title text-2xl font-bold justify-center mb-4">"Login"</h1>
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
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn AdminLoginPage() -> impl IntoView {
    let password = RwSignal::new("".to_string());

    let do_login = move || {
        spawn_local_scoped(async move {
            let params = password.get_untracked();

            match api_post("/api/admin/login", &params).await {
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
            <div class="card w-96 bg-base-100 shadow-xl border-t-4 border-error">
                <div class="card-body">
                    <h1 class="card-title text-2xl font-bold justify-center mb-4 text-error">
                        "Admin Login"
                    </h1>
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
                                placeholder="Admin Password"
                                class="input input-bordered w-full"
                                autocomplete="current-password"
                                on:input=move |ev| password.set(event_target_value(&ev))
                                prop:value=password
                                required
                            />
                        </div>
                        <div class="card-actions justify-end mt-6">
                            <button type="submit" class="btn btn-error w-full">
                                "Login"
                            </button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn StaffLoginPage() -> impl IntoView {
    let password = RwSignal::new("".to_string());

    let do_login = move || {
        spawn_local_scoped(async move {
            let params = password.get_untracked();

            match api_post("/api/staff/login", &params).await {
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
            <div class="card w-96 bg-base-100 shadow-xl border-t-4 border-secondary">
                <div class="card-body">
                    <h1 class="card-title text-2xl font-bold justify-center mb-4 text-secondary">
                        "Staff Login"
                    </h1>
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
                                placeholder="Staff Password"
                                class="input input-bordered w-full"
                                autocomplete="current-password"
                                on:input=move |ev| password.set(event_target_value(&ev))
                                prop:value=password
                                required
                            />
                        </div>
                        <div class="card-actions justify-end mt-6">
                            <button type="submit" class="btn btn-secondary w-full">
                                "Login"
                            </button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    }
}
