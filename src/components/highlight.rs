//! Renders clinical text with NLP entities highlighted.

use crate::domain::nlp::{Entity, EntityType};
use leptos::prelude::*;

#[component]
pub fn HighlightedText(#[prop(into)] text: Signal<String>, #[prop(into)] entities: Signal<Vec<Entity>>, #[prop(into, optional)] hidden: Signal<Vec<EntityType>>) -> impl IntoView {
    view! {
        <p class="whitespace-pre-wrap">
            {move || {
                let text = text.get();
                let hidden = hidden.get();
                let mut parts: Vec<AnyView> = Vec::new();
                let mut last = 0usize;
                for e in entities.get() {
                    if hidden.contains(&e.kind) || e.start < last || e.end > text.len() {
                        continue;
                    }
                    parts.push(text[last..e.start].to_string().into_any());
                    let title = format!(
                        "{}{}{}",
                        e.kind.label(),
                        if e.negated { " (negated)" } else { "" },
                        e.normalized.as_ref().map(|n| format!(" · {n}")).unwrap_or_default()
                    );
                    let class = format!("ent {} {} bg-transparent text-inherit", e.kind.swatch(), if e.negated { "ent-negated" } else { "" });
                    parts.push(view! { <mark class=class title=title>{text[e.start..e.end].to_string()}</mark> }.into_any());
                    last = e.end;
                }
                parts.push(text[last..].to_string().into_any());
                parts
            }}
        </p>
    }
}
