use crate::components::brand::{BrandMark, DemoPill};
use crate::components::browser;
use crate::components::icons::Icon;
use crate::components::smart_intake::Rec;
use crate::components::ui::{BtnVariant, Button, Field, INPUT_CLS};
use leptos::prelude::*;

#[derive(Clone, Default, PartialEq)]
struct Form {
    name: String,
    dob: String,
    phone: String,
    email: String,
    carrier: String,
    member_id: String,
    language: String,
}

fn empty() -> Form {
    Form { language: "English".into(), ..Default::default() }
}

#[component]
pub fn PortalPage() -> impl IntoView {
    let rec = RwSignal::new(None::<Rec>);
    let form = RwSignal::new(empty());
    let npp = RwSignal::new(false);
    let sms = RwSignal::new(false);
    let done = RwSignal::new(false);
    let timed_out = RwSignal::new(false);
    let idle = browser::use_idle(15 * 60 * 1000);

    Effect::new(move |_| {
        if let Some(raw) = browser::session_get("mos-intake") {
            if let Ok(r) = serde_json::from_str::<Rec>(&raw) {
                rec.set(Some(r));
            }
        }
    });

    // Automatic logoff: wipe everything typed after 15 minutes of inactivity.
    Effect::new(move |_| {
        if !idle.get() {
            return;
        }
        form.set(empty());
        npp.set(false);
        sms.set(false);
        timed_out.set(true);
        browser::session_remove("mos-intake");
        idle.set(false);
    });

    let valid = move || {
        form.with(|f| f.name.trim().chars().count() > 2 && !f.dob.is_empty() && f.phone.chars().filter(|c| c.is_ascii_digit()).count() >= 10) && npp.get()
    };

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if !valid() {
            return;
        }
        // Demo: nothing is transmitted or stored. Clear the data from memory and session storage.
        form.set(empty());
        npp.set(false);
        sms.set(false);
        browser::session_remove("mos-intake");
        done.set(true);
    };

    macro_rules! input {
        ($field:ident, $ty:expr) => {
            view! {
                <input
                    type=$ty
                    class=INPUT_CLS
                    prop:value=move || form.with(|f| f.$field.clone())
                    on:input=move |ev| {
                        let v = event_target_value(&ev);
                        form.update(|f| f.$field = v);
                    }
                    autocomplete="off"
                />
            }
        };
    }

    view! {
        <div class="min-h-screen bg-sand/60">
            <header class="border-b border-line bg-white">
                <div class="mx-auto flex max-w-5xl items-center justify-between gap-4 px-4 py-3 sm:px-6">
                    <a href="/" class="flex items-center gap-2.5">
                        <BrandMark uid="portal" class="size-9"/>
                        <span class="leading-tight">
                            <span class="block font-display text-[15px] font-semibold text-ink">"Meridian Orthopedics"</span>
                            <span class="flex items-center gap-1 text-[11px] font-semibold text-sage">
                                <Icon name="Lock" class="size-3"/>
                                " Secure Patient Portal"
                            </span>
                        </span>
                    </a>
                    <div class="flex items-center gap-3">
                        <span class="hidden font-mono text-[11px] text-muted sm:inline">"portal.meridian-ortho.example"</span>
                        <DemoPill class="!text-gold"/>
                    </div>
                </div>
            </header>

            <main class="mx-auto grid max-w-5xl gap-6 px-4 py-10 sm:px-6 lg:grid-cols-[1.3fr_1fr]">
                <section class="rounded-3xl border border-line bg-white p-6 shadow-[var(--shadow-soft)] sm:p-8">
                    <Show
                        when=move || done.get()
                        fallback=move || {
                            view! {
                                <form on:submit=submit autocomplete="off" novalidate>
                                    <h1 class="font-display text-2xl font-semibold text-ink">"Confirm your appointment request"</h1>
                                    <p class="mt-1 text-[14px] text-muted">
                                        "This is the only place we ask for personal details. The connection is encrypted and the page is never cached."
                                    </p>
                                    <Show when=move || timed_out.get()>
                                        <div class="mt-4 flex items-start gap-2 rounded-xl bg-warn-soft p-3 text-[12.5px] text-warn" role="status">
                                            <Icon name="Timer" class="mt-0.5 size-4 shrink-0"/>
                                            " For your privacy, the form was cleared after 15 minutes of inactivity."
                                        </div>
                                    </Show>

                                    <div class="mt-6 grid gap-4 sm:grid-cols-2">
                                        <Field label="Full legal name">{input!(name, "text")}</Field>
                                        <Field label="Date of birth">{input!(dob, "date")}</Field>
                                        <Field label="Mobile phone">
                                            <input
                                                type="tel"
                                                inputmode="tel"
                                                class=INPUT_CLS
                                                prop:value=move || form.with(|f| f.phone.clone())
                                                on:input=move |ev| {
                                                    let v = event_target_value(&ev);
                                                    form.update(|f| f.phone = v);
                                                }
                                                autocomplete="off"
                                                placeholder="(516) 555-0100"
                                            />
                                        </Field>
                                        <Field label="Email (optional)">{input!(email, "email")}</Field>
                                        <Field label="Insurance carrier">{input!(carrier, "text")}</Field>
                                        <Field label="Member ID" hint="Masked on screen; encrypted at rest">{input!(member_id, "password")}</Field>
                                        <Field label="Preferred language">
                                            <select
                                                class=INPUT_CLS
                                                prop:value=move || form.with(|f| f.language.clone())
                                                on:change=move |ev| {
                                                    let v = event_target_value(&ev);
                                                    form.update(|f| f.language = v);
                                                }
                                            >
                                                <option>"English"</option>
                                                <option>"Español"</option>
                                            </select>
                                        </Field>
                                    </div>

                                    <div class="mt-6 space-y-3 rounded-2xl bg-cream p-4 text-[13px]">
                                        <label class="flex cursor-pointer items-start gap-2.5">
                                            <input
                                                type="checkbox"
                                                class="mt-0.5 accent-[#1f5c8f]"
                                                prop:checked=move || npp.get()
                                                on:change=move |ev| npp.set(event_target_checked(&ev))
                                            />
                                            <span class="text-ink">
                                                "I acknowledge the "
                                                <a href="/privacy" class="font-semibold text-ocean underline">"Notice of Privacy Practices"</a>
                                                ". "
                                                <span class="text-danger">"*"</span>
                                            </span>
                                        </label>
                                        <label class="flex cursor-pointer items-start gap-2.5">
                                            <input
                                                type="checkbox"
                                                class="mt-0.5 accent-[#1f5c8f]"
                                                prop:checked=move || sms.get()
                                                on:change=move |ev| sms.set(event_target_checked(&ev))
                                            />
                                            <span class="text-muted">
                                                "Text me appointment reminders. Messages never include diagnoses or test results. Reply STOP to opt out. (Optional)"
                                            </span>
                                        </label>
                                    </div>

                                    <Button submit=true variant=BtnVariant::Sage class="mt-6 w-full" disabled=Signal::derive(move || !valid())>
                                        <Icon name="Lock" class="size-4"/>
                                        " Submit securely"
                                    </Button>
                                </form>
                            }
                        }
                    >
                        <div class="animate-fade-up py-8 text-center">
                            <div class="mx-auto grid size-14 place-items-center rounded-full bg-sage-soft text-sage">
                                <Icon name="CheckCircle2" class="size-7"/>
                            </div>
                            <h1 class="mt-4 font-display text-2xl font-semibold text-ink">"Request received"</h1>
                            <p class="mx-auto mt-2 max-w-md text-[14px] text-muted">
                                "We'll text you to confirm the time and send your forms in your preferred language."
                            </p>
                            <div class="mx-auto mt-5 max-w-md rounded-xl bg-gold-soft p-4 text-left text-[12.5px] text-ink">
                                <b>"Demo:"</b>
                                " nothing you typed was transmitted or stored, and the form has been cleared. In production this form posts over TLS to a HIPAA-compliant portal vendor under a Business Associate Agreement, and the data goes straight to the EMR, never into website logs, analytics or email."
                            </div>
                            <a href="/" class="mt-6 inline-flex items-center gap-1.5 text-[13px] font-semibold text-ocean hover:underline">
                                <Icon name="ArrowLeft" class="size-4"/>
                                " Back to the website"
                            </a>
                        </div>
                    </Show>
                </section>

                <aside class="space-y-4">
                    <div class="rounded-3xl border border-line bg-white p-6 shadow-[var(--shadow-soft)]">
                        <div class="text-[12px] font-semibold uppercase tracking-wide text-sage">"Your visit"</div>
                        {move || match rec.get() {
                            Some(r) => view! {
                                <div class="mt-1 font-display text-lg font-semibold text-ink">{r.service.clone()}</div>
                                <div class="text-[13px] text-muted">{r.who.clone()} " · " {r.where_.clone()}</div>
                                <div class="mt-3 inline-flex items-center gap-2 rounded-lg bg-sky-2 px-3 py-1.5 text-[13px] font-semibold text-ink">
                                    <Icon name="CalendarCheck2" class="size-4 text-sage"/>
                                    " "
                                    {r.when.clone()}
                                </div>
                                <ul class="mt-4 space-y-1.5 text-[12.5px] text-muted">
                                    {r
                                        .bring
                                        .iter()
                                        .map(|b| view! { <li class="flex gap-2"><Icon name="CheckCircle2" class="mt-0.5 size-3.5 shrink-0 text-sage"/>{b.clone()}</li> })
                                        .collect_view()}
                                </ul>
                            }
                            .into_any(),
                            None => view! {
                                <p class="mt-2 text-[13px] text-muted">
                                    "No visit selected yet. "
                                    <a href="/#request" class="font-semibold text-ocean">"Start a smart appointment request"</a>
                                    "."
                                </p>
                            }
                            .into_any(),
                        }}
                    </div>
                    <div class="rounded-3xl bg-ink p-6 text-white">
                        <div class="flex items-center gap-2 text-[13px] font-semibold">
                            <Icon name="ShieldCheck" class="size-4 text-gold-2"/>
                            " How this page protects you"
                        </div>
                        <ul class="mt-3 space-y-2 text-[12.5px] text-white/70">
                            {[
                                "Separated from the marketing site, which collects no personal data",
                                "Encrypted in transit (TLS) and at rest (AES-256)",
                                "Hosted by a vendor that signs a Business Associate Agreement",
                                "No analytics, ad pixels or chat widgets on this page",
                                "Never cached; clears itself after 15 minutes idle",
                                "Every access is written to an audit log",
                            ]
                                .into_iter()
                                .map(|t| view! { <li class="flex gap-2"><Icon name="KeyRound" class="mt-0.5 size-3.5 shrink-0 text-sage-2"/>{t}</li> })
                                .collect_view()}
                        </ul>
                    </div>
                </aside>
            </main>
        </div>
    }
}
