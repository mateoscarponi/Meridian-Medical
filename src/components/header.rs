use super::brand::{DemoPill, Logo};
use super::icons::Icon;
use super::ui::cx;
use crate::domain::practice::PRACTICE;
use leptos::prelude::*;

const NAV: [(&str, &str); 5] = [
    ("/#specialties", "Specialties"),
    ("/#conditions", "Conditions"),
    ("/#team", "Our Team"),
    ("/#locations", "Locations"),
    ("/#patients", "For Patients"),
];

#[component]
pub fn SiteHeader() -> impl IntoView {
    let open = RwSignal::new(false);
    let scrolled = RwSignal::new(false);
    Effect::new(move |_| {
        let on = move || scrolled.set(super::browser::window().scroll_y().unwrap_or(0.0) > 8.0);
        on();
        let off = super::browser::on_window("scroll", on);
        super::browser::cleanup(off);
    });

    view! {
        <header class="sticky top-0 z-50">
            <div class="bg-ink text-[12px] text-white/75">
                <div class="mx-auto flex max-w-7xl items-center justify-between gap-4 px-4 py-2 sm:px-6">
                    <div class="flex items-center gap-2.5">
                        <DemoPill/>
                        <span class="hidden sm:inline">"Serving " {PRACTICE.region} " since " {PRACTICE.since}</span>
                    </div>
                    <div class="flex items-center gap-4">
                        <a href="/#locations" class="hidden hover:text-white md:inline">
                            "Hicksville Walk-in Injury Clinic · No appointment needed"
                        </a>
                        <a href="tel:5165550148" class="inline-flex items-center gap-1.5 font-semibold text-white">
                            <Icon name="Phone" class="size-3.5"/>
                            " "
                            {PRACTICE.phone}
                        </a>
                    </div>
                </div>
            </div>
            <div class=move || {
                cx(&[
                    "border-b transition-all duration-300",
                    if scrolled.get() {
                        "border-line bg-cream/85 shadow-[0_8px_30px_-20px_rgb(15_30_46/0.35)] backdrop-blur-xl"
                    } else {
                        "border-transparent bg-cream"
                    },
                ])
            }>
                <div class="mx-auto flex max-w-7xl items-center justify-between gap-6 px-4 py-3.5 sm:px-6">
                    <Logo uid="hdr"/>
                    <nav class="hidden items-center gap-1 whitespace-nowrap xl:flex" aria-label="Primary">
                        {NAV
                            .iter()
                            .map(|(href, label)| {
                                view! {
                                    <a
                                        href=*href
                                        class="rounded-lg px-3 py-2 text-[14px] font-medium text-muted transition-colors hover:bg-sand hover:text-ink"
                                    >
                                        {*label}
                                    </a>
                                }
                            })
                            .collect_view()}
                        <a
                            href="/platform"
                            class="ml-1 inline-flex items-center gap-1 rounded-lg px-3 py-2 text-[14px] font-semibold text-ocean transition-colors hover:bg-sky-2"
                        >
                            "Care Platform "
                            <Icon name="ArrowUpRight" class="size-3.5"/>
                        </a>
                    </nav>
                    <div class="hidden items-center gap-2 whitespace-nowrap xl:flex">
                        <a
                            href="/#request"
                            class="rounded-xl bg-navy px-4 py-2.5 text-[14px] font-semibold text-white shadow-sm transition-all hover:bg-ocean active:scale-[0.98]"
                        >
                            "Request appointment"
                        </a>
                    </div>
                    <button
                        class="rounded-lg p-2 text-ink xl:hidden"
                        on:click=move |_| open.update(|v| *v = !*v)
                        aria-label=move || if open.get() { "Close menu" } else { "Open menu" }
                        aria-expanded=move || open.get().to_string()
                    >
                        {move || if open.get() { view! { <Icon name="X" class="size-6"/> } } else { view! { <Icon name="Menu" class="size-6"/> } }}
                    </button>
                </div>
                <Show when=move || open.get()>
                    <div class="border-t border-line bg-cream px-4 pb-5 pt-2 xl:hidden">
                        {NAV
                            .iter()
                            .map(|(href, label)| {
                                view! {
                                    <a
                                        href=*href
                                        on:click=move |_| open.set(false)
                                        class="block rounded-lg px-3 py-3 text-[15px] font-medium text-ink hover:bg-sand"
                                    >
                                        {*label}
                                    </a>
                                }
                            })
                            .collect_view()}
                        <a href="/platform" class="block rounded-lg px-3 py-3 text-[15px] font-semibold text-ocean hover:bg-sky-2">
                            "Care Platform →"
                        </a>
                        <a
                            href="/#request"
                            on:click=move |_| open.set(false)
                            class="mt-2 block rounded-xl bg-navy px-4 py-3 text-center text-[15px] font-semibold text-white"
                        >
                            "Request appointment"
                        </a>
                    </div>
                </Show>
            </div>
        </header>
    }
}
