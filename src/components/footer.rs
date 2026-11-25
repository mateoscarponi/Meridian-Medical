use super::brand::{DemoPill, Logo};
use super::icons::Icon;
use crate::domain::practice::PRACTICE;
use leptos::prelude::*;

#[component]
pub fn SiteFooter() -> impl IntoView {
    view! {
        <footer class="border-t border-line bg-cream">
            <div class="mx-auto grid max-w-7xl gap-10 px-4 py-14 sm:px-6 md:grid-cols-5">
                <div class="md:col-span-2">
                    <Logo uid="ftr"/>
                    <p class="mt-4 max-w-sm text-[14px] text-muted">
                        {PRACTICE.tagline} " Orthopedic and spine care for " {PRACTICE.region} " since " {PRACTICE.since} "."
                    </p>
                    <div class="mt-5 flex flex-wrap gap-2 text-[11.5px] font-semibold text-muted">
                        <span class="inline-flex items-center gap-1 rounded-full bg-white px-2.5 py-1 ring-1 ring-line">
                            <Icon name="Lock" class="size-3 text-sage"/>
                            " HTTPS only"
                        </span>
                        <span class="inline-flex items-center gap-1 rounded-full bg-white px-2.5 py-1 ring-1 ring-line">
                            <Icon name="ShieldCheck" class="size-3 text-sage"/>
                            " No ad trackers or cookies"
                        </span>
                        <span class="inline-flex items-center gap-1 rounded-full bg-white px-2.5 py-1 ring-1 ring-line">
                            <Icon name="ShieldCheck" class="size-3 text-sage"/>
                            " PHI only in the secure portal"
                        </span>
                    </div>
                </div>
                <div>
                    <div class="text-[13px] font-semibold text-ink">"Practice"</div>
                    <ul class="mt-3 space-y-2 text-[14px] text-muted">
                        <li><a href="/#specialties" class="hover:text-ink">"Specialties"</a></li>
                        <li><a href="/#team" class="hover:text-ink">"Our team"</a></li>
                        <li><a href="/#locations" class="hover:text-ink">"Locations"</a></li>
                        <li><a href="/#patients" class="hover:text-ink">"For patients"</a></li>
                    </ul>
                </div>
                <div>
                    <div class="text-[13px] font-semibold text-ink">"Patients"</div>
                    <ul class="mt-3 space-y-2 text-[14px] text-muted">
                        <li><a href="/portal" class="hover:text-ink">"Secure patient portal"</a></li>
                        <li><a href="/privacy" class="hover:text-ink">"Privacy & HIPAA"</a></li>
                        <li><a href="/privacy#notice" class="hover:text-ink">"Notice of Privacy Practices"</a></li>
                        <li><a href="/privacy#rights" class="hover:text-ink">"Your health information rights"</a></li>
                    </ul>
                </div>
                <div>
                    <div class="text-[13px] font-semibold text-ink">"Care platform"</div>
                    <ul class="mt-3 space-y-2 text-[14px] text-muted">
                        <li><a href="/platform" class="hover:text-ink">"Command center"</a></li>
                        <li><a href="/platform/impact" class="hover:text-ink">"Impact calculator"</a></li>
                        <li><a href="/platform/compliance" class="hover:text-ink">"HIPAA & security"</a></li>
                        <li><a href="/platform/architecture" class="hover:text-ink">"Architecture"</a></li>
                    </ul>
                </div>
            </div>
            <div class="border-t border-line">
                <div class="mx-auto max-w-7xl px-4 py-6 sm:px-6">
                    <div class="flex flex-col gap-3 rounded-2xl border border-line bg-white/70 p-4 sm:flex-row sm:items-center sm:justify-between">
                        <div class="flex items-start gap-3">
                            <DemoPill class="mt-0.5 !text-gold"/>
                            <p class="text-[12.5px] leading-relaxed text-muted">
                                <b class="text-ink">"Concept demo: not a real medical practice."</b>
                                " Built to show how AI can streamline a multi-site medical practice and optimize its workflows. Provider names, phone numbers and all patient data are fictional or synthetic; no real patients."
                            </p>
                        </div>
                        <div class="flex shrink-0 items-center gap-3">
                            <span class="text-[12px] text-faint">"Designed & engineered by Built by Oak · 2026"</span>
                            <a
                                href="https://www.linkedin.com/in/mateogscarponi/"
                                target="_blank"
                                rel="noopener noreferrer"
                                aria-label="LinkedIn profile (opens in a new tab)"
                                title="LinkedIn"
                                class="grid size-8 place-items-center rounded-lg bg-white text-muted ring-1 ring-line transition hover:bg-[#0a66c2] hover:text-white hover:ring-[#0a66c2]"
                            >
                                <svg viewBox="0 0 24 24" class="size-4" fill="currentColor" aria-hidden="true">
                                    <path d="M20.45 20.45h-3.55v-5.57c0-1.33-.03-3.04-1.85-3.04-1.86 0-2.14 1.45-2.14 2.94v5.67H9.36V9h3.41v1.56h.05c.48-.9 1.64-1.85 3.37-1.85 3.6 0 4.27 2.37 4.27 5.46v6.28zM5.34 7.43a2.06 2.06 0 1 1 0-4.12 2.06 2.06 0 0 1 0 4.12zM7.12 20.45H3.56V9h3.56v11.45zM22.22 0H1.77C.79 0 0 .77 0 1.73v20.54C0 23.23.79 24 1.77 24h20.45c.98 0 1.78-.77 1.78-1.73V1.73C24 .77 23.2 0 22.22 0z"></path>
                                </svg>
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        </footer>
    }
}
