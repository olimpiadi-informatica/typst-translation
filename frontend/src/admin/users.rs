use common::admin::{
    AddUserLanguageRequest, AdminUserOverview, AdminUserOverviewResponse, ImportUsersRequest,
    SetAllBudgetsRequest, SetBudgetRequest, UpdateContestantRequest, UpdatePasswordsJsonlRequest,
};
use common::contestant::Contestant;
use js_sys::Uint8Array;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use leptos::{ev, html};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlInputElement;

use crate::api_wrapper::{api_get, api_post};
use crate::util::{Card, Icon};
use crate::{show_error, show_success};

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

    let (jsonl_content, set_jsonl_content) = signal(String::new());
    let (all_budgets_val, set_all_budgets_val) = signal("1.00".to_string());

    let do_update_passwords = move |_| {
        let jsonl = jsonl_content.get();
        if jsonl.is_empty() {
            return;
        }
        spawn_local_scoped(async move {
            match api_post(
                "/api/admin/users/update_passwords",
                &UpdatePasswordsJsonlRequest {
                    jsonl_content: jsonl,
                },
            )
            .await
            {
                Ok(()) => {
                    users_resource.refetch();
                    set_jsonl_content.set(String::new());
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
            spawn_local_scoped(async move {
                let bytes: Uint8Array =
                    JsFuture::from(file.bytes()).await.unwrap().unchecked_into();
                let content = String::from_utf8(bytes.to_vec()).unwrap();
                set_jsonl_content.set(content);
            });
        }
    };

    let bulk_import_input_ref: NodeRef<html::Input> = NodeRef::new();
    let bulk_import_loading = RwSignal::new(false);

    let do_bulk_import = move || {
        spawn_local_scoped(async move {
            bulk_import_loading.set(true);
            let files = bulk_import_input_ref
                .get_untracked()
                .and_then(|input| input.files())
                .expect("No files selected");

            if let Some(file) = files.get(0) {
                let bytes: Uint8Array =
                    JsFuture::from(file.bytes()).await.unwrap().unchecked_into();
                let jsonl_content = String::from_utf8(bytes.to_vec()).unwrap();

                let payload = ImportUsersRequest { jsonl_content };
                match api_post("/api/admin/users/import", &payload).await {
                    Ok(()) => {
                        show_success!("Bulk import successful!");
                        users_resource.refetch();
                    }
                    Err(e) => {
                        show_error!("Bulk import failed: {e}");
                    }
                }
            }
            bulk_import_loading.set(false);
        });
    };

    view! {
        <div class="flex flex-col gap-8">
            <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
                <Card title="Update Passwords (JSONL)">
                    <div class="flex flex-col gap-4">
                        <p class="text-sm opacity-70">"Format: {\"username\": \"...\", \"password\": \"...\"}"</p>
                        <div class="flex gap-4 items-center">
                            <div class="join">
                                <input
                                    type="file"
                                    class="file-input file-input-bordered w-full max-w-xs join-item"
                                    on:change=on_file_change
                                />
                                <button
                                    class="btn btn-primary join-item"
                                    disabled=move || jsonl_content.get().is_empty()
                                    on:click=do_update_passwords
                                >
                                    "Update"
                                </button>
                            </div>
                        </div>
                    </div>
                </Card>

                <Card title="Bulk Import Users (JSONL)">
                    <div class="flex flex-col gap-4">
                        <div class="text-xs space-y-2 opacity-80">
                            <p>"Import countries, their languages, and contestants in bulk."</p>
                            <ul class="list-disc list-inside space-y-1">
                                <li>"One JSON object per line"</li>
                                <li>
                                    <code class="text-primary">"username"</code>
                                    ": unique country login"
                                </li>
                                <li>
                                    <code class="text-primary">"languages"</code>
                                    ": array of codes (e.g. \"it_IT\", \"it_CH\")"
                                </li>
                                <li>
                                    <code class="text-primary">"contestants"</code>
                                    ": array of objects with \"name\", \"code\", and optional \"online_bit\" (boolean)"
                                </li>
                            </ul>
                        </div>
                        <div class="flex gap-4 items-center">
                            <div class="join">
                                <input
                                    type="file"
                                    class="file-input file-input-bordered w-full join-item"
                                    accept=".jsonl"
                                    node_ref=bulk_import_input_ref
                                />
                                <button
                                    class="btn btn-primary join-item"
                                    on:click=move |_| do_bulk_import()
                                    disabled=bulk_import_loading
                                >
                                    {move || {
                                        if bulk_import_loading.get() {
                                            view! { <span class="loading loading-spinner loading-xs"></span> }
                                                .into_any()
                                        } else {
                                            view! { "Import" }.into_any()
                                        }
                                    }}
                                </button>
                            </div>
                        </div>
                    </div>
                </Card>

                <Card title="Set Everyone's Budget">
                    <div class="flex flex-col gap-4">
                        <p class="text-sm opacity-70">"Set the translation budget for ALL users."</p>
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
                                    "Set"
                                </button>
                            </div>
                        </div>
                    </div>
                </Card>
            </div>

            <Card title="Users Overview">
                <div class="overflow-x-auto">
                    <table class="table w-full">
                        <thead>
                            <tr>
                                <th class="w-16"></th>
                                <th>"ID"</th>
                                <th>"Username"</th>
                                <th>"Password"</th>
                                <th>"Budget"</th>
                                <th>"Languages"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || users_resource.get().flatten().unwrap_or_default()
                                key=|u| u.user.id
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
fn UserRow(
    overview: AdminUserOverview,
    refetch: impl Fn() + Copy + Send + 'static,
) -> impl IntoView {
    let (overview, _) = signal(overview);
    let (new_lang_code, set_new_lang_code) = signal(String::new());
    let (new_budget_val, set_new_budget_val) = signal(format!(
        "{:.2}",
        (overview.get_untracked().user.automatic_translation_budget as f64) / 1e9
    ));
    let (expanded, set_expanded) = signal(false);

    let set_budget = move |_| {
        let user_id = overview.get_untracked().user.id;
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
        let user_id = overview.get_untracked().user.id;
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

    let budget_dollars = move || (overview.get().user.automatic_translation_budget as f64) / 1e9;
    let used_dollars = move || (overview.get().user.tokens_used as f64) / 1e9;

    view! {
        <tr class="hover:bg-base-200 transition-colors">
            <td>
                <button
                    class="btn btn-ghost btn-xs"
                    on:click=move |_| set_expanded.update(|e| *e = !*e)
                >
                    {move || {
                        if expanded.get() {
                            view! { <Icon icon=icondata::AiCaretDownOutlined /> }.into_any()
                        } else {
                            view! { <Icon icon=icondata::AiCaretRightOutlined /> }.into_any()
                        }
                    }}
                </button>
            </td>
            <td>{move || overview.get().user.id}</td>
            <td class="font-mono text-xs">{move || overview.get().user.username.clone()}</td>
            <td class="font-mono text-xs">{move || overview.get().user.password.clone()}</td>
            <td>
                <div class="flex flex-col gap-1">
                    <div class="text-[10px] flex justify-between">
                        <span>"Rem: $" {move || format!("{:.2}", budget_dollars())}</span>
                        <span class="opacity-50">"Used: $" {move || format!("{:.2}", used_dollars())}</span>
                    </div>
                    <div class="join">
                        <input
                            type="text"
                            class="input input-bordered input-sm w-24 join-item"
                            on:input=move |ev| set_new_budget_val.set(event_target_value(&ev))
                            prop:value=new_budget_val
                        />
                        <button class="btn btn-primary btn-sm join-item" on:click=set_budget>
                            "Set"
                        </button>
                    </div>
                </div>
            </td>
            <td>
                <div class="flex flex-wrap gap-1 max-w-xs">
                    <For each=move || overview.get().languages.clone() key=|l| l.id let(l)>
                        <div class="badge badge-outline badge-sm">{l.code}</div>
                    </For>
                </div>
                <div class="mt-2 flex gap-1">
                    <input
                        type="text"
                        placeholder="code"
                        class="input input-bordered input-sm w-20"
                        on:input=move |ev| set_new_lang_code.set(event_target_value(&ev))
                        prop:value=new_lang_code
                    />
                    <button class="btn btn-sm btn-ghost" on:click=add_language>
                        <Icon icon=icondata::AiPlusOutlined />
                    </button>
                </div>
            </td>
        </tr>
        {move || {
            if expanded.get() {
                Either::Left(
                    view! {
                        <tr class="bg-base-200/50">
                            <td colspan="6" class="p-4">
                                <div class="flex flex-col gap-4 max-w-5xl mx-auto">
                                    <div class="flex justify-between items-center">
                                        <h4 class="font-bold text-sm">"Contestants Management"</h4>
                                    </div>
                                    <div class="overflow-x-auto bg-base-100 rounded-lg border border-base-300 shadow-inner">
                                        <table class="table table-sm w-full">
                                            <thead>
                                                <tr>
                                                    <th class="w-24">"Code"</th>
                                                    <th>"Name"</th>
                                                    <th class="w-32">"Online"</th>
                                                    <th class="w-20">"Action"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                <For
                                                    each=move || overview.get().contestants.clone()
                                                    key=|c| c.id
                                                    let(c)
                                                >
                                                    <ContestantRow contestant=c refetch=refetch />
                                                </For>
                                            </tbody>
                                        </table>
                                    </div>
                                    {move || {
                                        if overview.get().contestants.is_empty() {
                                            view! {
                                                <p class="text-xs opacity-50 italic px-4">
                                                    "No contestants found for this user."
                                                </p>
                                            }
                                                .into_any()
                                        } else {
                                            ().into_any()
                                        }
                                    }}
                                </div>
                            </td>
                        </tr>
                    },
                )
            } else {
                Either::Right(())
            }
        }}
    }
}

#[component]
fn ContestantRow(
    contestant: Contestant,
    refetch: impl Fn() + Copy + Send + 'static,
) -> impl IntoView {
    let (code, set_code) = signal(contestant.code.clone());
    let (name, set_name) = signal(contestant.name.clone());
    let (online, set_online) = signal(contestant.online_bit);

    let (loading, set_loading) = signal(false);

    let do_update = move |_| {
        let payload = UpdateContestantRequest {
            id: contestant.id,
            code: code.get(),
            name: name.get(),
            online_bit: online.get(),
        };
        set_loading.set(true);
        spawn_local_scoped(async move {
            match api_post("/api/admin/contestant/update", &payload).await {
                Ok(()) => {
                    refetch();
                }
                Err(e) => {
                    show_error!("Failed to update contestant: {e}");
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <tr class="hover:bg-base-200 transition-colors">
            <td>
                <input
                    type="text"
                    class="input input-bordered input-sm w-full font-mono"
                    on:input=move |ev| set_code.set(event_target_value(&ev))
                    prop:value=code
                />
            </td>
            <td>
                <input
                    type="text"
                    class="input input-bordered input-sm w-full"
                    on:input=move |ev| set_name.set(event_target_value(&ev))
                    prop:value=name
                />
            </td>
            <td>
                <label class="label cursor-pointer justify-start gap-2 py-0">
                    <input
                        type="checkbox"
                        class="checkbox checkbox-primary checkbox-sm"
                        checked=online
                        on:change=move |ev| set_online.set(event_target_checked(&ev))
                    />
                    <span class="label-text-alt opacity-70">"Online"</span>
                </label>
            </td>
            <td>
                <button
                    class="btn btn-primary btn-sm w-full"
                    on:click=do_update
                    disabled=loading
                >
                    {move || if loading.get() { "..." } else { "Save" }}
                </button>
            </td>
        </tr>
    }
}
