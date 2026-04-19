use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Icon(#[prop(into)] icon: Signal<icondata::Icon>) -> impl IntoView {
    view! {
        <svg
            inner_html=move || icon.get().data
            viewBox=move || icon.get().view_box
            stroke-linecap=move || icon.get().stroke_linecap
            stroke-linejoin=move || icon.get().stroke_linejoin
            stroke-width=move || icon.get().stroke_width
            stroke=move || icon.get().stroke
            width="1em"
            height="1em"
            x=move || icon.get().x
            y=move || icon.get().y
            fill=move || icon.get().fill.unwrap_or("currentColor")
        />
    }
}

#[component]
pub fn Card(
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional, into)] header: Option<AnyView>,
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=move || {
            format!(
                "card bg-base-100 shadow-none border border-base-300 {}",
                class.as_ref().cloned().unwrap_or_default(),
            )
        }>
            <div class="card-body">
                {if let Some(header) = header {
                    header
                } else if let Some(title) = title {
                    view! { <h2 class="card-title mb-4">{title}</h2> }.into_any()
                } else {
                    ().into_any()
                }} {children()}
            </div>
        </div>
    }
}

#[component]
pub fn NavTabs(
    #[prop(into)] tabs: Signal<Vec<(String, String)>>, // (label, href)
) -> impl IntoView {
    let location = leptos_router::hooks::use_location();
    view! {
        <div role="tablist" class="tabs tabs-boxed">
            <For each=move || tabs.get() key=|(label, href)| format!("{}-{}", label, href) let(tab)>
                <A
                    href=tab.1.clone()
                    attr:class=move || {
                        if location.pathname.get() == tab.1 { "tab tab-active" } else { "tab" }
                    }
                >
                    {tab.0}
                </A>
            </For>
        </div>
    }
}
