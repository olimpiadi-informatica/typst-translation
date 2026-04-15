use common::admin::{
    AddUserLanguageRequest, AdminUserOverview, AdminUserOverviewResponse, SetAllBudgetsRequest,
    SetBudgetRequest, UpdatePasswordsCsvRequest,
};
use leptos::ev;
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{FileReader, HtmlInputElement};

use crate::api_wrapper::{api_get, api_post};
use crate::show_error;
use crate::util::{Card, Icon};

#[component]
pub fn AdminUsersPage() -> impl IntoView {
    let users_resource = LocalResource::new(|| async {
        match api_get::<AdminUserOverviewResponse>("/api/admin/users/overview").await {
            Ok(data) => Some(data),
            Err(e) => {
                show_error!("Failed to fetch users: {e}");
                None
            }
        }
    });

    let (csv_content, set_csv_content) = signal(String::new());
    let (all_budgets_val, set_all_budgets_val) = signal("1.00".to_string());

    let do_update_passwords = move |_| {
        let csv = csv_content.get();
        if csv.is_empty() {
            return;
        }
        spawn_local_scoped(async move {
            match api_post(
                "/api/admin/users/update_passwords",
                &UpdatePasswordsCsvRequest { csv_content: csv },
            )
            .await
            {
                Ok(()) => {
                    users_resource.refetch();
                    set_csv_content.set(String::new());
                }
                Err(e) => {
                    show_error!("Update failed: {e}");
                }
            }
        });
    };

    let set_all_budgets = move |_| {
        let budget_usd: f64 = all_budgets_val.get().parse().unwrap_or(0.0);
        let new_budget = (budget_usd * 1e9) as i64;
        spawn_local_scoped(async move {
            match api_post(
                "/api/admin/users/set_all_budgets",
                &SetAllBudgetsRequest { new_budget },
            )
            .await
            {
                Ok(()) => {
                    users_resource.refetch();
                }
                Err(e) => {
                    show_error!("Failed to set all budgets: {e}");
                }
            }
        });
    };

    let on_file_change = move |ev: ev::Event| {
        let target = ev.target().unwrap().dyn_into::<HtmlInputElement>().unwrap();
        let files = target.files().unwrap();
        if let Some(file) = files.get(0) {
            let reader = FileReader::new().unwrap();
            let reader_c = reader.clone();
            let onload = Closure::wrap(Box::new(move |_ev: web_sys::Event| {
                if let Ok(result) = reader_c.result()
                    && let Some(content) = result.as_string()
                {
                    set_csv_content.set(content);
                }
            }) as Box<dyn FnMut(_)>);
            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
            reader.read_as_text(&file).unwrap();
            onload.forget();
        }
    };

    view! {
        <div class="flex flex-col gap-8">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
                <Card title="Update Passwords (CSV)">
                    <div class="flex flex-col gap-4">
                        <p class="text-sm opacity-70">"Format: username,password"</p>
                        <div class="flex gap-4 items-center">
                            <div class="join">
                                <input
                                    type="file"
                                    class="file-input file-input-bordered w-full max-w-xs join-item"
                                    on:change=on_file_change
                                />
                                <button
                                    class="btn btn-primary join-item"
                                    disabled=move || csv_content.get().is_empty()
                                    on:click=do_update_passwords
                                >
                                    "Update"
                                </button>
                            </div>
                        </div>
                    </div>
                </Card>

                <Card title="Set Everyone's Budget">
                    <div class="flex flex-col gap-4">
                        <p class="text-sm opacity-70">
                            "Set the translation budget for ALL users."
                        </p>
                        <div class="flex gap-4 items-center">
                            <div class="join">
                                <span class="join-item btn btn-disabled no-animation">"$"</span>
                                <input
                                    type="text"
                                    class="input input-bordered join-item w-32"
                                    on:input=move |ev| {
                                        set_all_budgets_val.set(event_target_value(&ev))
                                    }
                                    prop:value=all_budgets_val
                                />
                                <button class="btn btn-primary join-item" on:click=set_all_budgets>
                                    "Set for All"
                                </button>
                            </div>
                        </div>
                    </div>
                </Card>
            </div>

            <Card title="Users Overview">
                <div class="overflow-x-auto">
                    <table class="table table-zebra w-full">
                        <thead>
                            <tr>
                                <th>"ID"</th>
                                <th>"Username"</th>
                                <th>"Password"</th>
                                <th>"Translation Budget"</th>
                                <th>"Languages"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || users_resource.get().flatten().unwrap_or_default()
                                key=|u| u.clone()
                                let(overview)
                            >
                                <UserRow
                                    overview
                                    refetch=move || {
                                        users_resource.refetch();
                                    }
                                />
                            </For>
                        </tbody>
                    </table>
                </div>
            </Card>
        </div>
    }
}

#[component]
fn UserRow(overview: AdminUserOverview, refetch: impl Fn() + Copy + 'static) -> impl IntoView {
    let (new_lang_code, set_new_lang_code) = signal(String::new());
    let (new_budget_val, set_new_budget_val) = signal(format!(
        "{:.2}",
        (overview.user.automatic_translation_budget as f64) / 1e9
    ));

    let set_budget = move |_| {
        let user_id = overview.user.id;
        let budget_usd: f64 = new_budget_val.get().parse().unwrap_or(0.0);
        let new_budget = (budget_usd * 1e9) as i64;
        spawn_local_scoped(async move {
            match api_post(
                "/api/admin/users/set_budget",
                &SetBudgetRequest {
                    user_id,
                    new_budget,
                },
            )
            .await
            {
                Ok(()) => {
                    refetch();
                }
                Err(e) => {
                    show_error!("Failed to set budget: {e}");
                }
            }
        });
    };

    let add_language = move |_| {
        let user_id = overview.user.id;
        let code = new_lang_code.get();
        if code.is_empty() {
            return;
        }
        spawn_local_scoped(async move {
            match api_post(
                "/api/admin/users/add_language",
                &AddUserLanguageRequest {
                    user_id,
                    language_code: code,
                },
            )
            .await
            {
                Ok(()) => {
                    refetch();
                    set_new_lang_code.set(String::new());
                }
                Err(e) => {
                    show_error!("Add language failed: {e}");
                }
            }
        });
    };

    let budget_dollars = move || (overview.user.automatic_translation_budget as f64) / 1e9;
    let used_dollars = move || (overview.user.tokens_used as f64) / 1e9;

    view! {
        <tr>
            <td>{overview.user.id}</td>
            <td class="font-mono text-xs">{overview.user.username.clone()}</td>
            <td class="font-mono text-xs">{overview.user.password.clone()}</td>
            <td>
                <div class="flex flex-col gap-2">
                    <div class="text-xs">
                        <div class="flex flex-col">
                            <span>"Rem: $" {move || format!("{:.2}", budget_dollars())}</span>
                            <span class="opacity-50 text-[10px]">
                                "Used: $" {move || format!("{:.2}", used_dollars())}
                            </span>
                        </div>
                    </div>
                    <div class="join">
                        <span class="join-item btn btn-xs btn-disabled no-animation">"$"</span>
                        <input
                            type="text"
                            class="input input-bordered input-xs w-16 join-item"
                            on:input=move |ev| set_new_budget_val.set(event_target_value(&ev))
                            prop:value=new_budget_val
                        />
                        <button class="btn btn-primary btn-xs join-item" on:click=set_budget>
                            "Set"
                        </button>
                    </div>
                </div>
            </td>
            <td>
                <div class="flex flex-wrap gap-1 max-w-xs">
                    <For each=move || overview.languages.clone() key=|l| l.id let(l)>
                        <div class="badge badge-outline badge-sm">{l.code}</div>
                    </For>
                </div>
                <div class="mt-2 flex gap-1">
                    <input
                        type="text"
                        placeholder="code"
                        class="input input-bordered input-xs w-16"
                        on:input=move |ev| set_new_lang_code.set(event_target_value(&ev))
                        prop:value=new_lang_code
                    />
                    <button class="btn btn-xs btn-ghost" on:click=add_language>
                        <Icon icon=icondata::AiPlusOutlined />
                    </button>
                </div>
            </td>
        </tr>
    }
}
