use leptos::prelude::*;

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
