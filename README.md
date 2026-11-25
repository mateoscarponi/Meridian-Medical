# Meridian Orthopedics & Spine Care

**A concept website and AI-powered operations platform for a multi-site medical practice, built entirely in Rust.**

![Walkthrough: the practice website with its rotating 3D spine, its privacy-first appointment request and the secure patient portal, then the Care Operations Platform: the command center, the Prior Auth Copilot drafting and submitting a request, Document AI processing incoming faxes, the claim scrubber fixing errors, the AI front-desk agent booking a patient in Spanish, an HL7 message converted to FHIR, the impact calculator, and the live HIPAA scan](docs/meridian-demo-v3.gif)

**[Live demo](https://meridianmedical.vercel.app)** · [Migrated to Rust](#migrated-from-typescript-and-python-to-rust) · [Guided tour](#take-the-5-minute-tour) · [What success looks like](#what-success-looks-like) · [Run it locally](#run-it-locally)

> **Concept demo.** Meridian Orthopedics & Spine Care is not a real practice. Provider names and phone numbers are fictional, and every patient, claim and document in the platform is synthetic. No real patient data is used anywhere in this project.

---

## Migrated from TypeScript and Python to Rust

The project was first built with Next.js and React in **TypeScript**, plus a **Python** script to train the no-show model. It has since been **migrated completely to Rust**: the web server, every page and component, the API routes, the clinical logic, the 3D model on the home page and the machine-learning training. The original TypeScript version is preserved in the [`typescript-legacy`](https://github.com/mateoscarponi/Meridian-Medical/tree/typescript-legacy) branch and the [`v1-typescript`](https://github.com/mateoscarponi/Meridian-Medical/releases/tag/v1-typescript) tag.

**Same functionality, same look, no regressions.** The migration was verified against the original, not just eyeballed:

- **Logic parity:** the original TypeScript code and the new Rust code were run on the same inputs (clinical NLP on eight notes, PHI de-identification, HL7 → FHIR conversion of every sample message, prior-authorization criteria and letters, and the generated SOAP note). The outputs match field by field: **0 differences across 18 test cases**.
- **Visual parity:** all **633 Tailwind classes** of the original are present in the Rust build, with the same colors, fonts, spacing and animations. The visible text of all **21 pages** was compared word by word against the TypeScript site; the only changes are the ones that now say Rust instead of TypeScript or Python.
- **Model parity:** the no-show model trainer reproduces Python's random number generator exactly, so the Rust version produces **byte-identical model weights**.

**Faster.** Measured on a local machine with the release build:

| | Rust |
|---|---|
| Time to server-render a full page | **~1.3 ms** |
| Throughput | **~3,000 full pages per second** (16 concurrent clients) |
| FHIR API response | ~0.5 ms |
| Server memory | ~18 MB |
| Training the no-show model (Applied AI) | **0.3 seconds**, compared with **~21 seconds** in Python (about 70× faster), with identical results |

The whole browser bundle (all 21 pages) is a single WebAssembly module of about 580 KB compressed, downloaded once; after that, moving between pages downloads nothing more.

## Why this project exists

Behind every patient visit there is a long trail of administrative work. Front-desk teams verify insurance by phone, authorization specialists rebuild the same paperwork for every payer, faxes are retyped into the medical record by hand, and billing teams fix claim errors only after a payer has already denied them. In a practice with several locations, that work multiplies, and it takes time away from patients.

This project shows, end to end, how software and applied AI can take much of that burden off staff and providers. It is modeled on a multi-site orthopedic and spine practice across Nassau and Suffolk counties on Long Island, a setting with heavy authorization, workers' compensation and no-fault volume. The same approach applies to almost any medical practice.

It has two parts:

1. **A practice website** that is calm, fast and easy to use, with a smart appointment request that sends each patient to the right visit without collecting any personal information.
2. **A Care Operations Platform** with eighteen working tools. Each one targets a specific administrative workflow and reports the time it saves.

## What success looks like

The platform is built around five outcomes. Every tool exists to move at least one of them, and each one can be measured.

| Outcome | How the platform delivers it | Tools |
|---|---|---|
| **Reduced administrative workload for staff and providers** | Repetitive work becomes review work. Faxes are read and routed automatically, insurance is verified overnight, routine patient questions are answered by an AI agent, and visit notes are drafted for the provider to sign. | Document AI · Eligibility · AI Front-Desk Agent · Patient Messages · Clinical Scribe |
| **Faster processing of authorizations and documentation** | The Prior Auth Copilot reads the chart, checks each payer criterion with cited evidence, fills the questionnaire, drafts the letter of medical necessity and submits it. A request that took about 45 minutes takes about 4. It also flags requests that would be denied, before anyone sends them. | Prior Auth Copilot · Clinical Scribe · Chart NLP · WC & No-Fault Desk |
| **Improved interoperability between systems** | A working integration layer converts HL7 v2 messages from the medical record into FHIR R4 resources, serves a live FHIR API, and models the payer transactions (X12) and EMR connections a practice depends on. | HL7 / FHIR Lab · Data Pipeline · AI Agent (reads and writes through the FHIR API) |
| **Increased operational efficiency through automation** | Event-driven workflows run across the medical record, payers, fax and text messaging, such as turning a fax referral into a booked appointment or a denial into a corrected claim. People approve only the steps that need judgment. | Automations · Referrals · Smart Scheduling · Claim Scrubber |
| **AI tools that measurably improve workflows** | Every tool shows its impact: hours returned, turnaround times, denial rates, no-show rates. The Command Center tracks each outcome from baseline to target, and the Impact Calculator turns a practice's own volumes into projected hours and dollars. | Command Center · Impact & ROI Calculator |

## Take the 5-minute tour

The quickest way to see the project is the [live demo](https://meridianmedical.vercel.app):

1. **[Website](https://meridianmedical.vercel.app):** answer the smart appointment request and continue to the secure patient portal.
2. **[Command Center](https://meridianmedical.vercel.app/platform):** the five outcomes, work queues and AI-recommended actions.
3. **[Prior Auth Copilot](https://meridianmedical.vercel.app/platform/prior-auth):** press *Run Copilot* on the first request, then try *Carmen Diaz* to see a request the Copilot holds back.
4. **[Document AI](https://meridianmedical.vercel.app/platform/documents):** press *Process all* and watch each document get classified, extracted and routed.
5. **[AI Front-Desk Agent](https://meridianmedical.vercel.app/platform/agent):** play the Spanish rescheduling conversation and follow every tool call in the trace.
6. **[HL7 / FHIR Lab](https://meridianmedical.vercel.app/platform/interop):** send a real HL7 message to the server and inspect the FHIR result.
7. **[Impact & ROI](https://meridianmedical.vercel.app/platform/impact):** enter a practice's own numbers.

## What's inside

### The practice website

- Specialties, conditions treated, care philosophy, team, four locations, and a visit checklist with accepted insurance.
- A **rotating 3D model of the spine**, rendered with WebGL2 directly from Rust: procedural geometry and physically based shading, drawn in 2 draw calls per frame. It pauses when off screen and respects reduced-motion settings.
- **Smart appointment request:** four quick questions route the patient to the right specialty, provider and location, and list what to bring for their type of coverage. Emergency symptoms stop the flow and direct the patient to emergency care.
- **Secure patient portal:** the only place personal details are entered. It is separated from the marketing site, never cached, and clears itself after 15 minutes of inactivity.
- **Privacy & HIPAA page** in plain language, with a Notice of Privacy Practices summary and patient rights.

### The Care Operations Platform

| Area | Tool | What it does |
|---|---|---|
| Overview | **Command Center** | Live scorecard for the five outcomes, team work queues with service levels, integration health, and AI-recommended next actions. |
| | **Impact & ROI Calculator** | Editable practice volumes and automation levers, turned into staff hours, provider hours, recovered visits and annual impact. |
| Revenue cycle | **Prior Auth Copilot** | Chart evidence extraction, payer criteria with cited proof, letter of medical necessity, and submission tracking. It covers commercial plans, Medicare Advantage (including the federal 7-day decision rule) and New York workers' compensation guidelines. |
| | **Eligibility & Benefits** | Overnight verification of the next day's schedule, so staff start the day with exceptions only: terminated plans, expected balances, pending authorizations. |
| | **Claim Scrubber** | Checks claims before submission for bundling errors, missing modifiers, retired diagnosis codes, missing authorizations and New York no-fault and workers' comp rules, with a live denial-risk score and one-click fixes. |
| | **WC & No-Fault Desk** | Tracks strict New York filing deadlines (initial reports, no-fault applications and bills) and drafts the forms from the chart. |
| Front office | **Smart Scheduling** | A no-show prediction model scores every appointment, explains its reasons, recommends the lightest intervention that works, and backfills open slots from the waitlist. |
| | **Referral Tracker** | One pipeline for referrals arriving by fax, secure message, FHIR or portal, with automations and loop closure back to the referring physician. |
| | **Patient Messages** | One inbox for texts, portal messages and voicemail. It classifies intent, escalates emergencies first, and drafts replies in the patient's language for staff approval. |
| | **AI Front-Desk Agent** | A tool-using agent that verifies identity, looks up the patient through the live FHIR API, reschedules, sends secure links and hands anything clinical to a human. |
| Clinical AI | **Clinical Scribe** | Turns a visit transcript into a structured SOAP note with diagnosis-code suggestions, coding support, bilingual patient instructions, and follow-up actions that start when the provider signs. |
| | **Chart NLP** | Extracts symptoms, spinal levels, imaging findings, medications and conservative-care duration from free text, detects negation ("denies fever") and red flags, and suggests diagnosis codes. |
| | **Document AI Inbox** | Classifies incoming faxes and uploads, extracts the key fields, matches the patient, routes each document to the right queue and writes the result back as FHIR. |
| Data & integration | **HL7 / FHIR Lab** | Parses any HL7 v2 message field by field, converts it to a FHIR R4 bundle, returns the acknowledgment, and lets you query the live FHIR API. |
| | **Data Pipeline** | A cloud-agnostic pipeline (AWS, Azure or Google Cloud) with standard terminology mapping, data-quality checks, a warehouse query, and patient matching across systems. |
| | **Automations** | Workflow recipes with triggers, conditions and human approval steps. Each simulated run shows exactly what happened and how long it took. |
| Trust & build | **HIPAA & Security** | A live compliance scan of the deployment, a Business Associate Agreement register, Security Rule safeguards, PHI de-identification, role-based access and an audit trail. |
| | **Architecture** | Reference architecture, reliability targets, technology stack, a 90-day rollout plan and excerpts from the code. |

## Key capabilities

**Healthcare interoperability.** HL7 v2.5.1 parsing and acknowledgments, FHIR R4 resources aligned with US Core, a FHIR API with a capability statement, and models of the X12 payer transactions (eligibility, authorization, claims, remittance). It also shows how each major EMR connects: eClinicalWorks, Epic, athenahealth, NextGen, Oracle Health (Cerner), AdvancedMD and DrChrono.

**Applied AI and machine learning.**
- Large language model integration (Claude) for clinical notes and patient replies, with an on-device fallback so the demo always works.
- Clinical language processing with negation and red-flag detection, compiled to WebAssembly so it runs in the browser and no patient text leaves the device.
- A no-show prediction model trained in Rust (0.3 seconds) and scored live in the browser.
- An AI agent that uses real tools against the FHIR API, inside clear guardrails.

**Revenue cycle knowledge.** Prior authorization criteria and evidence, diagnosis and procedure coding (ICD-10-CM, CPT), bundling and modifier rules, denial prevention, and New York workers' compensation and no-fault requirements.

**Automation and data engineering.** Event-driven workflows with retries and human approval steps, an extract-transform-load pipeline with quality checks and quarantine, cross-system patient matching, and a star-schema warehouse for reporting.

**Security and compliance.** HIPAA safeguards are part of the design from the start rather than added later, as described below.

## Privacy & HIPAA by design

Even as a demo, the project follows the practices a real medical website and platform need:

- **Personal data stays out of the website.** The appointment request collects no names, contact details or insurance IDs. Identifiers are entered only in the secure portal, and the hand-off between them never places health information in a web address.
- **Encrypted, hardened delivery.** Every page is served with strict security headers: forced HTTPS, a content security policy with a fresh nonce on every page so only the site's own code can run, protection against being embedded in other sites, and blocked camera, microphone and location access. Pages and APIs that carry patient data are never cached.
- **No tracking technologies.** No advertising pixels, session recording, third-party chat widgets or tracking cookies. Fonts are self-hosted.
- **Minimum necessary for AI.** Direct identifiers are removed from any text before it is sent to the language model, even though the model provider would be covered by a Business Associate Agreement.
- **Accountability.** Server routes write audit events that record who did what and when, never the patient data itself. The platform locks automatically after 15 minutes of inactivity.
- **Verifiable.** The HIPAA & Security page runs a live scan of the deployment in your browser and shows the result.

## Run it locally

Requirements: [Rust](https://rustup.rs) (stable), the WebAssembly target and `wasm-bindgen-cli` (its version must match the `wasm-bindgen` crate in `Cargo.lock`):

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130 --locked
```

Build and run:

```bash
bash scripts/build.sh
./target/release/meridian_medical
```

Open http://localhost:3018 for the website and http://localhost:3018/platform for the platform.

`scripts/build.sh` compiles the browser bundle (Rust → WebAssembly), generates its JavaScript bindings, copies the static assets and compiles the server. On Windows it uses `scripts/env.sh` to work around MinGW tooling issues with spaces in the user folder.

**Optional language model.** Set the `ANTHROPIC_API_KEY` environment variable (see `.env.example`) to have the Clinical Scribe and Patient Messages generate text with Claude. Without a key, both use their built-in generators.

**Retrain the no-show model:**

```bash
cargo run --release --bin train_noshow
```

This writes the learned weights to `data/no-show-weights.json`, which is embedded in the scheduling tool at compile time.

**Tests and parity checks:**

```bash
cargo test
node scripts/parity-ts.mjs <path-to-typescript-version>   # runs the original TypeScript logic
cargo test --test parity && python scripts/compare-parity.py
```

**Styles.** `public/main.css` is generated from the Tailwind classes in the Rust sources with `node scripts/build-css.mjs <path-to-node_modules-with-tailwind>`; it only needs regenerating after changing classes.

### Deployment

The site runs on Vercel with Vercel's native Rust runtime. `api/vercel.rs` runs the same Axum application as the local server inside a Vercel Function, and `vercel.json` routes every request to it. The WebAssembly bundle, stylesheet and fonts are prebuilt into `public/` and served from Vercel's CDN.

### Public API endpoints

| Endpoint | Purpose |
|---|---|
| `GET /api/fhir/metadata` | FHIR capability statement |
| `GET /api/fhir/Patient?name=` · `/api/fhir/Patient/{id}` | Patient search and read |
| `GET /api/fhir/Coverage?patient=` · `/Appointment?date=` · `/ServiceRequest` · `/Practitioner` | Other FHIR resources |
| `POST /api/hl7` | Send a raw HL7 v2 message; returns the acknowledgment and the FHIR bundle |
| `POST /api/ai` | Clinical note or patient reply generation (language model when configured) |

## Built with

- **Rust** end to end: **Axum** web server, **Leptos** components rendered on the server and hydrated in the browser as **WebAssembly**, and **WebGL2** for the 3D spine
- **Tailwind CSS 4** design tokens, self-hosted fonts
- **Claude API** for the language model integration
- Healthcare standards: **HL7 v2**, **FHIR R4**, **X12 5010**, **ICD-10-CM**, **CPT**, **LOINC**
- Originally built with Next.js 15, React 19, TypeScript and Python (see the [`typescript-legacy`](https://github.com/mateoscarponi/Meridian-Medical/tree/typescript-legacy) branch)

## Project structure

```
src/main.rs        Local web server
api/vercel.rs      Vercel Function entry point (same application)
src/app.rs         HTML shell and page routes
src/pages/         Website, patient portal, privacy page and the eighteen platform tools
src/components/    Shared interface components, site and platform shells, charts, 3D spine
src/domain/        HL7 parser and FHIR mapping, clinical NLP, PHI redaction, claim rules,
                   prior-auth criteria, scheduling model, synthetic data
src/server/        API routes, security headers, audit logging
src/bin/           No-show model training
public/            Stylesheet, fonts, icon and the prebuilt WebAssembly bundle
tests/, scripts/   Parity checks against the original version, build and style scripts
```

## Limitations

- All data is synthetic, and integrations with EMRs, payers, fax and text messaging are simulated. The HL7 parser, FHIR API, clinical NLP, claim rules, PHI redaction and no-show model run for real.
- Payer criteria and coding rules are illustrative composites, not any payer's proprietary policy. Regulatory references reflect public guidance at the time of writing; always confirm current requirements.
- The project is a technology demonstration. It does not provide medical, legal or billing advice, and it is not affiliated with any medical practice.

## License

[MIT](LICENSE)
