//! Scroll-reveal wrapper: fades/slides content in the first time it enters the viewport.

use super::ui::cx;
use leptos::html::Div;
use leptos::prelude::*;

#[component]
pub fn Reveal(#[prop(optional, into)] class: String, #[prop(optional)] delay: u32, children: Children) -> impl IntoView {
    let node = NodeRef::<Div>::new();
    Effect::new(move |_| {
        if let Some(el) = node.get() {
            let target: web_sys::Element = el.clone().into();
            let io = super::browser::observe(&target, 0.12, "0px 0px -40px 0px", move |visible, io| {
                if visible {
                    let _ = el.class_list().add_1("is-visible");
                    io.disconnect();
                }
            });
            super::browser::cleanup(move || io.disconnect());
        }
    });
    view! {
        <div node_ref=node class=cx(&["reveal", &class]) style=format!("transition-delay:{delay}ms")>
            {children()}
        </div>
    }
}
