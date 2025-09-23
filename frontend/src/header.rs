use common::user::ExtUser;
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use leptos_use::ColorMode;
use strum::VariantArray;
use thaw::{Button, ButtonAppearance, Flex, Icon, Select, Text};

use crate::api_wrapper::api_post;
use crate::editor::KeyboardMode;
use crate::user::UserContext;
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
    #[prop(name = "color_mode")] (color_mode, set_color_mode): SignalPair<ColorMode>,
    #[prop(name = "kb_mode")] (kb_mode, set_kb_mode): SignalPair<KeyboardMode>,
) -> impl IntoView {
    let name_and_icon = Signal::derive(move || {
        if color_mode.get() == ColorMode::Light {
            ("Dark", icondata::BiMoonSolid)
        } else {
            ("Light", icondata::BiSunSolid)
        }
    });
    let change_theme = move |_| {
        if color_mode.get() == ColorMode::Dark {
            set_color_mode.set(ColorMode::Light)
        } else {
            set_color_mode.set(ColorMode::Dark)
        }
    };

    let kb_mode_str = Signal::derive(move || select_kb_mode_str(kb_mode.get()).to_string());
    let set_kb_mode_str = SignalSetter::map(move |s: String| {
        set_kb_mode.set(kb_mode_from_str(&s));
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
                let user_context = expect_context::<UserContext>();
                user_context.refetch();
            })
        })
    };

    view! {
        <Flex>
            <Button on_click=change_theme appearance=ButtonAppearance::Subtle>
                {move || {
                    let (name, icon) = name_and_icon.get();
                    view! {
                        <Icon icon style="padding: 0 5px 0 0;" width="1.5em" height="1.5em" />
                        <Text>{name}</Text>
                    }
                }}
            </Button>
            <Select value=(kb_mode_str, set_kb_mode_str)>
                <For each=move || KeyboardMode::VARIANTS key=|x| *x let(v)>
                    <option>{move || select_kb_mode_str(*v)}</option>
                </For>
            </Select>
            <p>
                {
                    let user_context = expect_context::<UserContext>();
                    match user_context.get_user() {
                        ExtUser::User(user) => user.username,
                        _ => todo!(),
                    }
                }
            </p>
            <Button on_click=do_logout>"Logout"</Button>
        </Flex>
    }
}
