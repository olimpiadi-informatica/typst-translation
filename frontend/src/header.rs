use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use leptos_router::hooks::use_navigate;
use strum::VariantArray;

use crate::api_wrapper::api_post;
use crate::editor::KeyboardMode;
use crate::user::ExtUserContext;
use crate::{show_error, show_success};

type SignalPair<T> = (Signal<T>, WriteSignal<T>);

fn select_kb_mode_str(kb_mode: KeyboardMode) -> &'static str {
    match kb_mode {
        KeyboardMode::Vim => "Vim mode",
        KeyboardMode::Emacs => "Emacs mode",
        KeyboardMode::Standard => "Standard mode",
    }
}

fn kb_mode_from_str(s: &str) -> KeyboardMode {
    KeyboardMode::VARIANTS
        .iter()
        .copied()
        .find(|x| select_kb_mode_str(*x) == s)
        .unwrap()
}

#[component]
pub fn Header(
    #[prop(optional, into)] left_action: Option<AnyView>,
    #[prop(optional, into)] title: Option<Signal<String>>,
    #[prop(optional)] tabs: Option<AnyView>,
    #[prop(optional)] kb_mode: Option<SignalPair<KeyboardMode>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let kb_mode_view = kb_mode.map(|(kb_mode, set_kb_mode)| {
        view! {
            <select
                class="select select-bordered select-sm"
                on:change=move |ev| {
                    set_kb_mode.set(kb_mode_from_str(&event_target_value(&ev)));
                }
            >
                <For
                    each=move || KeyboardMode::VARIANTS
                    key=|x| *x
                    let(v)
                >
                    <option selected=move || kb_mode.get() == *v>
                        {move || select_kb_mode_str(*v)}
                    </option>
                </For>
            </select>
        }
    });

    let owner = Owner::current().unwrap();
    let do_logout = move |_| {
        owner.with(move || {
            spawn_local_scoped(async move {
                match api_post("/api/logout", &()).await {
                    Ok(()) => {}
                    Err(e) => {
                        show_error!("Failed to logout: {e}");
                        return;
                    }
                }

                show_success!("Logout successful");
                let user_context = expect_context::<ExtUserContext>();
                user_context.refetch();
                let navigate = use_navigate();
                navigate("/", Default::default());
            })
        })
    };

    let user_context = expect_context::<ExtUserContext>();

    view! {
        <div class="navbar bg-base-100 shadow-sm px-4 h-16 flex justify-between items-center">
            <div class="navbar-start flex items-center gap-2">
                {left_action}
                {move || {
                    title
                        .get()
                        .map(|title| {
                            view! { <h1 class="text-xl font-bold">{title}</h1> }
                        })
                }}
                {kb_mode_view}
            </div>
            <div class="navbar-center hidden lg:flex">
                {tabs}
            </div>
            <div class="navbar-end flex items-center gap-4">
                {children.map(|c| c())}
                <div class="flex items-center gap-2">
                    <Show when=move || user_context.get_ext_user_untracked().user.is_some()>
                        <p class="text-sm">"User: "</p>
                        <code class="bg-base-200 px-2 py-1 rounded">
                            {move || user_context.get_user_untracked().username}
                        </code>
                    </Show>
                    <button class="btn btn-primary btn-sm" on:click=do_logout>
                        "Logout"
                    </button>
                </div>
            </div>
        </div>
    }
}
