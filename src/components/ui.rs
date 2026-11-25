//! Shared UI primitives (badges, cards, buttons, stats, progress, code blocks, segmented control).
//! Class strings are identical to the React originals so the compiled Tailwind CSS matches 1:1.

use leptos::prelude::*;

pub use crate::domain::util::money;

/// Joins class names, skipping empty ones (`cx` in the original).
pub fn cx(parts: &[&str]) -> String {
    parts.iter().filter(|p| !p.is_empty()).copied().collect::<Vec<_>>().join(" ")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Tone {
    #[default]
    Neutral,
    Ocean,
    Sage,
    Gold,
    Danger,
    Warn,
    Ink,
}

impl Tone {
    pub fn classes(self) -> &'static str {
        match self {
            Tone::Neutral => "bg-sand text-muted ring-line",
            Tone::Ocean => "bg-sky-2 text-ocean ring-sky",
            Tone::Sage => "bg-sage-soft text-sage ring-sage/20",
            Tone::Gold => "bg-gold-soft text-gold ring-gold/20",
            Tone::Danger => "bg-danger-soft text-danger ring-danger/20",
            Tone::Warn => "bg-warn-soft text-warn ring-warn/20",
            Tone::Ink => "bg-ink text-white ring-ink",
        }
    }
}

#[component]
pub fn Badge(#[prop(into, optional)] tone: Signal<Tone>, #[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <span class=move || {
            cx(&["inline-flex items-center gap-1 whitespace-nowrap rounded-full px-2 py-0.5 text-[11px] font-semibold ring-1 ring-inset", tone.get().classes(), &class])
        }>{children()}</span>
    }
}

#[component]
pub fn Card(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! { <div class=cx(&["rounded-2xl border border-line bg-white shadow-[var(--shadow-soft)]", &class])>{children()}</div> }
}

#[component]
pub fn CardHeader(
    #[prop(into)] title: Signal<String>,
    #[prop(optional, into)] subtitle: Option<Signal<String>>,
    #[prop(optional, into)] right: Option<ViewFn>,
    #[prop(optional, into)] icon: Option<ViewFn>,
) -> impl IntoView {
    view! {
        <div class="flex items-start justify-between gap-3 border-b border-line px-5 py-4">
            <div class="flex min-w-0 items-start gap-3">
                {icon.map(|i| view! { <div class="mt-0.5 grid size-8 shrink-0 place-items-center rounded-lg bg-sky-2 text-ocean">{i.run()}</div> })}
                <div class="min-w-0">
                    <h3 class="text-[15px] font-semibold text-ink">{move || title.get()}</h3>
                    {subtitle.map(|s| view! { <p class="mt-0.5 text-[13px] text-muted">{move || s.get()}</p> })}
                </div>
            </div>
            {right.map(|r| view! { <div class="shrink-0">{r.run()}</div> })}
        </div>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BtnVariant {
    #[default]
    Primary,
    Secondary,
    Ghost,
    Sage,
    Dark,
}

impl BtnVariant {
    pub fn classes(self) -> &'static str {
        match self {
            BtnVariant::Primary => "bg-ocean text-white hover:bg-navy shadow-sm",
            BtnVariant::Sage => "bg-sage text-white hover:bg-[#25724f] shadow-sm",
            BtnVariant::Dark => "bg-ink text-white hover:bg-ink-2 shadow-sm",
            BtnVariant::Secondary => "bg-white text-ink ring-1 ring-inset ring-line-2 hover:bg-sand",
            BtnVariant::Ghost => "text-ocean hover:bg-sky-2",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BtnSize {
    Sm,
    #[default]
    Md,
}

#[component]
pub fn Button(
    #[prop(optional)] variant: BtnVariant,
    #[prop(optional)] size: BtnSize,
    #[prop(optional, into)] class: String,
    #[prop(into, optional)] disabled: Signal<bool>,
    #[prop(optional)] submit: bool,
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional, into)] on_click: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    let classes = cx(&[
        "inline-flex items-center justify-center gap-2 rounded-xl font-semibold transition-all duration-200 disabled:cursor-not-allowed disabled:opacity-50 active:scale-[0.98]",
        if size == BtnSize::Sm { "px-3 py-1.5 text-[13px]" } else { "px-4 py-2.5 text-sm" },
        variant.classes(),
        &class,
    ]);
    view! {
        <button
            type={if submit { "submit" } else { "button" }}
            class=classes
            title=title
            disabled=move || disabled.get()
            on:click=move |_| {
                if let Some(cb) = on_click {
                    cb.run(());
                }
            }
        >
            {children()}
        </button>
    }
}

#[component]
pub fn LinkButton(href: &'static str, #[prop(optional)] variant: BtnVariant, #[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <a
            href=href
            class=cx(&["inline-flex items-center justify-center gap-2 rounded-xl px-5 py-3 text-sm font-semibold transition-all duration-200 active:scale-[0.98]", variant.classes(), &class])
        >
            {children()}
        </a>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StatTone {
    #[default]
    Sage,
    Danger,
    Neutral,
}

#[component]
pub fn Stat(
    label: &'static str,
    #[prop(into)] value: Signal<String>,
    #[prop(into, optional)] delta: Signal<Option<String>>,
    #[prop(optional)] tone: StatTone,
    #[prop(into, optional)] hint: Option<String>,
) -> impl IntoView {
    let delta_cls = match tone {
        StatTone::Sage => "text-[12px] font-semibold text-sage",
        StatTone::Danger => "text-[12px] font-semibold text-danger",
        StatTone::Neutral => "text-[12px] font-semibold text-muted",
    };
    view! {
        <Card class="p-4">
            <div class="text-[12px] font-medium uppercase tracking-wide text-faint">{label}</div>
            <div class="mt-1.5 flex items-baseline gap-2">
                <div class="whitespace-nowrap font-display text-[28px] leading-none font-semibold text-ink tabular-nums">{move || value.get()}</div>
                {move || delta.get().map(|d| view! { <span class=delta_cls>{d}</span> })}
            </div>
            {hint.map(|h| view! { <div class="mt-1.5 text-[12px] text-muted">{h}</div> })}
        </Card>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ProgressTone {
    #[default]
    Ocean,
    Sage,
    Gold,
    Danger,
    Warn,
}

#[component]
pub fn Progress(#[prop(into)] value: Signal<f64>, #[prop(into, optional)] tone: Signal<ProgressTone>, #[prop(optional, into)] class: String) -> impl IntoView {
    let color = move || match tone.get() {
        ProgressTone::Ocean => "bg-ocean",
        ProgressTone::Sage => "bg-sage",
        ProgressTone::Gold => "bg-gold",
        ProgressTone::Danger => "bg-danger",
        ProgressTone::Warn => "bg-warn",
    };
    view! {
        <div
            class=cx(&["h-1.5 w-full overflow-hidden rounded-full bg-sand", &class])
            role="progressbar"
            aria-valuenow=move || crate::domain::util::js_round(value.get()).to_string()
            aria-valuemin="0"
            aria-valuemax="100"
        >
            <div
                class=move || cx(&["h-full rounded-full transition-[width] duration-700 ease-out", color()])
                style:width=move || format!("{}%", crate::domain::util::num(value.get().clamp(0.0, 100.0)))
            ></div>
        </div>
    }
}

#[component]
pub fn Eyebrow(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! { <div class=cx(&["text-[12px] font-semibold uppercase tracking-[0.16em] text-gold", &class])>{children()}</div> }
}

#[component]
pub fn CodeBlock(#[prop(into)] value: Signal<String>, #[prop(optional, into)] class: String, #[prop(optional)] max_h: Option<&'static str>) -> impl IntoView {
    view! {
        <pre class=cx(&[
            "scroll-thin overflow-auto rounded-xl bg-ink p-4 font-mono text-[12px] leading-relaxed text-[#cfe1ee]",
            max_h.unwrap_or("max-h-96"),
            &class,
        ])>
            <code>{move || value.get()}</code>
        </pre>
    }
}

/// `JSON.stringify(value, null, 2)`
pub fn pretty_json(v: &serde_json::Value) -> String {
    serde_json::to_string_pretty(v).unwrap_or_default()
}

#[component]
pub fn Segmented<T>(value: RwSignal<T>, options: Vec<(T, &'static str)>) -> impl IntoView
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    view! {
        <div class="inline-flex rounded-xl bg-sand p-1 ring-1 ring-inset ring-line">
            {options
                .into_iter()
                .map(|(v, label)| {
                    let v2 = v.clone();
                    view! {
                        <button
                            on:click=move |_| value.set(v.clone())
                            class=move || {
                                cx(&[
                                    "rounded-lg px-3 py-1.5 text-[13px] font-semibold transition-all",
                                    if value.with(|x| *x == v2) { "bg-white text-ink shadow-sm" } else { "text-muted hover:text-ink" },
                                ])
                            }
                        >
                            {label}
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}

#[component]
pub fn Field(label: &'static str, #[prop(optional)] hint: Option<&'static str>, children: Children) -> impl IntoView {
    view! {
        <label class="block">
            <span class="mb-1.5 block text-[12px] font-semibold text-muted">{label}</span>
            {children()}
            {hint.map(|h| view! { <span class="mt-1 block text-[11px] text-faint">{h}</span> })}
        </label>
    }
}

pub const INPUT_CLS: &str = "w-full rounded-xl border border-line-2 bg-white px-3 py-2 text-sm text-ink outline-none transition focus:border-ocean focus:ring-4 focus:ring-sky";
