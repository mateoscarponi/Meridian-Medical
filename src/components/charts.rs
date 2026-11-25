//! Lightweight SVG/HTML charts with hover tooltips.
//! Validated 2-series palette (dataviz validator, light surface): ocean #1F5C8F + green #2E9466.

use crate::domain::util::{js_round, num};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

pub const C_A: &str = "#1F5C8F";
pub const C_B: &str = "#2E9466";

#[derive(Clone)]
pub struct LegendItem {
    pub label: String,
    pub color: &'static str,
    pub dashed: bool,
}

#[component]
pub fn Legend(items: Vec<LegendItem>) -> impl IntoView {
    view! {
        <div class="flex flex-wrap gap-4 text-[12px] text-muted">
            {items
                .into_iter()
                .map(|i| {
                    view! {
                        <span class="inline-flex items-center gap-1.5">
                            {if i.dashed {
                                view! { <span class="h-0 w-4 border-t-2 border-dashed" style=format!("border-color:{}", i.color)></span> }.into_any()
                            } else {
                                view! { <span class="size-2.5 rounded-[3px]" style=format!("background:{}", i.color)></span> }.into_any()
                            }}
                            {i.label}
                        </span>
                    }
                })
                .collect_view()}
        </div>
    }
}

/// Horizontal paired bars: "before" (A) vs "now" (B) per category.
#[component]
pub fn PairedBars(data: Vec<(&'static str, f64, f64)>, a_label: &'static str, b_label: &'static str, #[prop(optional)] unit: &'static str) -> impl IntoView {
    let hover = RwSignal::new(None::<usize>);
    let max = data.iter().flat_map(|d| [d.1, d.2]).fold(f64::MIN, f64::max) * 1.08;
    let op = move |i: usize| if hover.get().is_none_or(|h| h == i) { "1" } else { "0.35" };
    view! {
        <div>
            <Legend items=vec![
                LegendItem { label: a_label.into(), color: C_A, dashed: false },
                LegendItem { label: b_label.into(), color: C_B, dashed: false },
            ]/>
            <div class="mt-4 space-y-3">
                {data
                    .into_iter()
                    .enumerate()
                    .map(|(i, (label, a, b))| {
                        view! {
                            <div
                                class="relative grid grid-cols-[120px_1fr] items-center gap-3 sm:grid-cols-[150px_1fr]"
                                on:mouseenter=move |_| hover.set(Some(i))
                                on:mouseleave=move |_| hover.set(None)
                            >
                                <div class="truncate text-[12px] font-medium text-ink">{label}</div>
                                <div class="space-y-[2px] py-1">
                                    <div class="flex items-center gap-2">
                                        <div
                                            class="h-2.5 rounded-r-[4px] transition-all duration-700"
                                            style:width=format!("{}%", num(a / max * 100.0))
                                            style:background=C_A
                                            style:opacity=move || op(i)
                                        ></div>
                                        <span class="text-[11px] tabular-nums text-muted">{num(a)} {unit}</span>
                                    </div>
                                    <div class="flex items-center gap-2">
                                        <div
                                            class="h-2.5 rounded-r-[4px] transition-all duration-700"
                                            style:width=format!("{}%", num(b / max * 100.0))
                                            style:background=C_B
                                            style:opacity=move || op(i)
                                        ></div>
                                        <span class="text-[11px] font-semibold tabular-nums text-ink">{num(b)} {unit}</span>
                                    </div>
                                </div>
                                <Show when=move || hover.get() == Some(i)>
                                    <div class="pointer-events-none absolute right-0 top-0 z-10 rounded-lg bg-ink px-2.5 py-1.5 text-[11px] text-white shadow-lg">
                                        {label} ": " {num(a)} {unit} " → " {num(b)} {unit} " "
                                        <b class="text-[#8fd6b1]">"−" {num(js_round((1.0 - b / a) * 100.0))} "%"</b>
                                    </div>
                                </Show>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}

/// Single-series line with optional target line, crosshair + tooltip on hover.
#[component]
pub fn TrendLine(
    points: Vec<(String, f64)>,
    #[prop(optional)] unit: &'static str,
    #[prop(optional)] target: Option<f64>,
    #[prop(optional)] target_label: Option<&'static str>,
    #[prop(optional)] height: Option<f64>,
    label: &'static str,
) -> impl IntoView {
    let hover = RwSignal::new(None::<usize>);
    let w = 560.0;
    let h = height.unwrap_or(180.0);
    let (pl, pr, pt, pb) = (34.0, 12.0, 12.0, 26.0);
    let n = points.len();
    let max = (points.iter().map(|p| p.1).chain(target).fold(f64::MIN, f64::max) * 1.15).ceil();
    let x = move |i: usize| pl + (i as f64 / (n - 1) as f64) * (w - pl - pr);
    let y = move |v: f64| pt + (1.0 - v / max) * (h - pt - pb);
    let path = points.iter().enumerate().map(|(i, p)| format!("{}{},{}", if i > 0 { "L" } else { "M" }, num(x(i)), num(y(p.1)))).collect::<Vec<_>>().join(" ");
    let ticks: Vec<f64> = [0.0, max / 2.0, max].iter().map(|t| js_round(t * 10.0) / 10.0).collect();
    let every = n.div_ceil(7);
    let pts = StoredValue::new(points.clone());
    let last_y = points.last().map(|p| p.1).unwrap_or(0.0);
    let target_label = target_label.unwrap_or("Target");

    let on_move = move |e: leptos::ev::MouseEvent| {
        let Some(el) = e.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) else { return };
        let r = el.get_bounding_client_rect();
        let px = (e.client_x() as f64 - r.left()) / r.width() * w;
        let i = js_round((px - pl) / (w - pl - pr) * (n - 1) as f64);
        hover.set(Some(i.clamp(0.0, (n - 1) as f64) as usize));
    };

    view! {
        <div class="relative">
            <svg viewBox=format!("0 0 {} {}", num(w), num(h)) class="w-full" role="img" aria-label=label on:mouseleave=move |_| hover.set(None) on:mousemove=on_move>
                {ticks
                    .iter()
                    .map(|t| {
                        view! {
                            <g>
                                <line x1=num(pl) x2=num(w - pr) y1=num(y(*t)) y2=num(y(*t)) stroke="#e6dfd3" stroke-width="1"></line>
                                <text x=num(pl - 6.0) y=num(y(*t) + 3.0) text-anchor="end" font-size="10" fill="#8a95a0">{num(*t)}</text>
                            </g>
                        }
                    })
                    .collect_view()}
                {target
                    .map(|t| {
                        view! {
                            <g>
                                <line x1=num(pl) x2=num(w - pr) y1=num(y(t)) y2=num(y(t)) stroke=C_B stroke-width="1.5" stroke-dasharray="5 4"></line>
                                <text x=num(w - pr) y=num(y(t) - 5.0) text-anchor="end" font-size="10" fill="#5b6875">{format!("{target_label} {}{unit}", num(t))}</text>
                            </g>
                        }
                    })}
                <path d=path fill="none" stroke=C_A stroke-width="2" stroke-linejoin="round" stroke-linecap="round"></path>
                {points
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        let show = i % every == 0 || i == n - 1;
                        view! {
                            <text x=num(x(i)) y=num(h - 8.0) text-anchor="middle" font-size="10" fill="#8a95a0">{if show { p.0.clone() } else { String::new() }}</text>
                        }
                    })
                    .collect_view()}
                <circle cx=num(x(n - 1)) cy=num(y(last_y)) r="4" fill=C_A stroke="#fff" stroke-width="2"></circle>
                {move || {
                    hover
                        .get()
                        .map(|i| {
                            let v = pts.with_value(|p| p[i].1);
                            view! {
                                <g>
                                    <line x1=num(x(i)) x2=num(x(i)) y1=num(pt) y2=num(h - pb) stroke="#5b6875" stroke-width="1" stroke-dasharray="2 3"></line>
                                    <circle cx=num(x(i)) cy=num(y(v)) r="4.5" fill=C_A stroke="#fff" stroke-width="2"></circle>
                                </g>
                            }
                        })
                }}
            </svg>
            {move || {
                hover
                    .get()
                    .map(|i| {
                        let (xl, v) = pts.with_value(|p| p[i].clone());
                        view! {
                            <div
                                class="pointer-events-none absolute top-0 rounded-lg bg-ink px-2.5 py-1.5 text-[11px] text-white shadow-lg"
                                style=format!("left: clamp(0px, calc({}% - 50px), calc(100% - 110px))", num(x(i) / w * 100.0))
                            >
                                {xl}
                                ": "
                                <b>{num(v)} {unit}</b>
                            </div>
                        }
                    })
            }}
        </div>
    }
}

/// Vertical single-series bars with hover.
#[component]
pub fn Bars(data: Vec<(&'static str, f64)>, #[prop(optional)] unit: &'static str, #[prop(optional)] color: Option<&'static str>, label: &'static str) -> impl IntoView {
    let hover = RwSignal::new(None::<usize>);
    let max = data.iter().map(|d| d.1).fold(f64::MIN, f64::max) * 1.12;
    let color = color.unwrap_or(C_A);
    view! {
        <div class="flex h-40 items-end gap-[2px]" role="img" aria-label=label>
            {data
                .into_iter()
                .enumerate()
                .map(|(i, (l, v))| {
                    view! {
                        <div
                            class="relative flex h-full flex-1 flex-col items-center justify-end"
                            on:mouseenter=move |_| hover.set(Some(i))
                            on:mouseleave=move |_| hover.set(None)
                        >
                            <Show when=move || hover.get() == Some(i)>
                                <div class="absolute -top-1 z-10 whitespace-nowrap rounded-md bg-ink px-2 py-1 text-[11px] text-white">
                                    {l} ": " {num(v)} {unit}
                                </div>
                            </Show>
                            <div
                                class="w-full max-w-9 rounded-t-[4px] transition-all duration-700"
                                style:height=format!("{}%", num(v / max * 100.0))
                                style:background=color
                                style:opacity=move || if hover.get().is_none_or(|h| h == i) { "1" } else { "0.4" }
                            ></div>
                            <div class="mt-1.5 w-full truncate text-center text-[10px] text-faint">{l}</div>
                        </div>
                    }
                })
                .collect_view()}
        </div>
    }
}
