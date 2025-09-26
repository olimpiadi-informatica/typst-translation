use common::user::ExtUser;
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use strum::VariantArray;
use thaw::{
    Button, ButtonAppearance, ButtonShape, ButtonSize, Flex, FlexAlign, FlexJustify, Select,
};

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
    #[prop(optional)] go_back: Option<String>,
    #[prop(optional, into)] title: Option<Signal<String>>,
    #[prop(optional)] kb_mode: Option<SignalPair<KeyboardMode>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    //let color_mode = use_color_mode_with_options(
    //    UseColorModeOptions::default()
    //        .cookie_enabled(true)
    //        .cookie_name("typst-translation-color-mode"),
    //);
    //let (color_mode, set_color_mode) = (color_mode.mode, color_mode.set_mode);

    //let name_and_icon = Signal::derive(move || {
    //    if color_mode.get() == ColorMode::Light {
    //        ("Dark", icondata::BiMoonSolid)
    //    } else {
    //        ("Light", icondata::BiSunSolid)
    //    }
    //});
    //let change_theme = move |_| {
    //    if color_mode.get_untracked() == ColorMode::Dark {
    //        set_color_mode.set(ColorMode::Light)
    //    } else {
    //        set_color_mode.set(ColorMode::Dark)
    //    }
    //};

    let kb_mode_view = kb_mode.map(|(kb_mode, set_kb_mode)| {
        let kb_mode_str = Signal::derive(move || select_kb_mode_str(kb_mode.get()).to_string());
        let set_kb_mode_str = SignalSetter::map(move |s: String| {
            set_kb_mode.set(kb_mode_from_str(&s));
        });

        view! {
            <Select value=(kb_mode_str, set_kb_mode_str)>
                <For each=move || KeyboardMode::VARIANTS key=|x| *x let(v)>
                    <option>{move || select_kb_mode_str(*v)}</option>
                </For>
            </Select>
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
                let user_context = expect_context::<UserContext>();
                user_context.refetch();
                let navigate = use_navigate();
                navigate("/", Default::default());
            })
        })
    };

    view! {
        <Flex justify=FlexJustify::SpaceBetween align=FlexAlign::Center style="height: 64px">
            <Flex align=FlexAlign::Center>
                {go_back
                    .map(|url| {
                        view! {
                            <A href=url>
                                <Button
                                    icon=icondata::BiArrowBackRegular
                                    appearance=ButtonAppearance::Subtle
                                    shape=ButtonShape::Circular
                                    size=ButtonSize::Large
                                />
                            </A>
                        }
                    })} // TODO: Fix theme handling
                {move || title.get().map(|title| view! { <h1>{title}</h1> })}
                // <Button on_click=change_theme appearance=ButtonAppearance::Subtle>
                // {move || {
                // let (name, icon) = name_and_icon.get();
                // view! {
                // <Icon icon style="padding: 0 5px 0 0;" width="1.5em" height="1.5em" />
                // <Text>{name}</Text>
                // }
                // }}
                // </Button>
                {kb_mode_view}
            </Flex>
            {children.map(|c| c())}
            <Flex>
                <p>"User: "</p>
                <pre>
                    {
                        let user_context = expect_context::<UserContext>();
                        match user_context.get_user_untracked() {
                            ExtUser::User(user) => user.username,
                            _ => todo!(),
                        }
                    }
                </pre>
                <Button on_click=do_logout>"Logout"</Button>
            </Flex>
        </Flex>
    }
}
