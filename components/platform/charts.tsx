"use client";

import { useState } from "react";

// Validated 2-series palette (dataviz validator, light surface): ocean #1F5C8F + green #2E9466.
const C_A = "#1F5C8F";
const C_B = "#2E9466";

export function Legend({ items }: { items: { label: string; color: string; dashed?: boolean }[] }) {
  return (
    <div className="flex flex-wrap gap-4 text-[12px] text-muted">
      {items.map((i) => (
        <span key={i.label} className="inline-flex items-center gap-1.5">
          {i.dashed ? <span className="h-0 w-4 border-t-2 border-dashed" style={{ borderColor: i.color }} /> : <span className="size-2.5 rounded-[3px]" style={{ background: i.color }} />}
          {i.label}
        </span>
      ))}
    </div>
  );
}

/** Horizontal paired bars: "before" (A) vs "now" (B) per category. */
export function PairedBars({ data, aLabel, bLabel, unit = "" }: { data: { label: string; a: number; b: number }[]; aLabel: string; bLabel: string; unit?: string }) {
  const [hover, setHover] = useState<number | null>(null);
  const max = Math.max(...data.flatMap((d) => [d.a, d.b])) * 1.08;
  return (
    <div>
      <Legend items={[{ label: aLabel, color: C_A }, { label: bLabel, color: C_B }]} />
      <div className="mt-4 space-y-3">
        {data.map((d, i) => (
          <div key={d.label} className="relative grid grid-cols-[120px_1fr] items-center gap-3 sm:grid-cols-[150px_1fr]" onMouseEnter={() => setHover(i)} onMouseLeave={() => setHover(null)}>
            <div className="truncate text-[12px] font-medium text-ink">{d.label}</div>
            <div className="space-y-[2px] py-1">
              <div className="flex items-center gap-2">
                <div className="h-2.5 rounded-r-[4px] transition-all duration-700" style={{ width: `${(d.a / max) * 100}%`, background: C_A, opacity: hover === null || hover === i ? 1 : 0.35 }} />
                <span className="text-[11px] tabular-nums text-muted">{d.a}{unit}</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="h-2.5 rounded-r-[4px] transition-all duration-700" style={{ width: `${(d.b / max) * 100}%`, background: C_B, opacity: hover === null || hover === i ? 1 : 0.35 }} />
                <span className="text-[11px] font-semibold tabular-nums text-ink">{d.b}{unit}</span>
              </div>
            </div>
            {hover === i && (
              <div className="pointer-events-none absolute right-0 top-0 z-10 rounded-lg bg-ink px-2.5 py-1.5 text-[11px] text-white shadow-lg">
                {d.label}: {d.a}{unit} → {d.b}{unit} <b className="text-[#8fd6b1]">−{Math.round((1 - d.b / d.a) * 100)}%</b>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

/** Single-series line with optional target line, crosshair + tooltip on hover. */
export function TrendLine({ points, unit = "", target, targetLabel = "Target", height = 180, label }: { points: { x: string; y: number }[]; unit?: string; target?: number; targetLabel?: string; height?: number; label: string }) {
  const [hover, setHover] = useState<number | null>(null);
  const W = 560;
  const H = height;
  const pad = { l: 34, r: 12, t: 12, b: 26 };
  const ys = points.map((p) => p.y).concat(target ?? []);
  const max = Math.ceil(Math.max(...ys) * 1.15);
  const x = (i: number) => pad.l + (i / (points.length - 1)) * (W - pad.l - pad.r);
  const y = (v: number) => pad.t + (1 - v / max) * (H - pad.t - pad.b);
  const path = points.map((p, i) => `${i ? "L" : "M"}${x(i)},${y(p.y)}`).join(" ");
  const ticks = [0, max / 2, max].map((t) => Math.round(t * 10) / 10);
  return (
    <div className="relative">
      <svg viewBox={`0 0 ${W} ${H}`} className="w-full" role="img" aria-label={label}
        onMouseLeave={() => setHover(null)}
        onMouseMove={(e) => {
          const r = (e.currentTarget as SVGSVGElement).getBoundingClientRect();
          const px = ((e.clientX - r.left) / r.width) * W;
          const i = Math.round(((px - pad.l) / (W - pad.l - pad.r)) * (points.length - 1));
          setHover(Math.max(0, Math.min(points.length - 1, i)));
        }}>
        {ticks.map((t) => (
          <g key={t}>
            <line x1={pad.l} x2={W - pad.r} y1={y(t)} y2={y(t)} stroke="#e6dfd3" strokeWidth={1} />
            <text x={pad.l - 6} y={y(t) + 3} textAnchor="end" fontSize={10} fill="#8a95a0">{t}</text>
          </g>
        ))}
        {target !== undefined && (
          <g>
            <line x1={pad.l} x2={W - pad.r} y1={y(target)} y2={y(target)} stroke={C_B} strokeWidth={1.5} strokeDasharray="5 4" />
            <text x={W - pad.r} y={y(target) - 5} textAnchor="end" fontSize={10} fill="#5b6875">{targetLabel} {target}{unit}</text>
          </g>
        )}
        <path d={path} fill="none" stroke={C_A} strokeWidth={2} strokeLinejoin="round" strokeLinecap="round" />
        {points.map((p, i) => (
          <text key={p.x} x={x(i)} y={H - 8} textAnchor="middle" fontSize={10} fill="#8a95a0">{i % Math.ceil(points.length / 7) === 0 || i === points.length - 1 ? p.x : ""}</text>
        ))}
        <circle cx={x(points.length - 1)} cy={y(points[points.length - 1].y)} r={4} fill={C_A} stroke="#fff" strokeWidth={2} />
        {hover !== null && (
          <g>
            <line x1={x(hover)} x2={x(hover)} y1={pad.t} y2={H - pad.b} stroke="#5b6875" strokeWidth={1} strokeDasharray="2 3" />
            <circle cx={x(hover)} cy={y(points[hover].y)} r={4.5} fill={C_A} stroke="#fff" strokeWidth={2} />
          </g>
        )}
      </svg>
      {hover !== null && (
        <div className="pointer-events-none absolute top-0 rounded-lg bg-ink px-2.5 py-1.5 text-[11px] text-white shadow-lg" style={{ left: `clamp(0px, calc(${(x(hover) / W) * 100}% - 50px), calc(100% - 110px))` }}>
          {points[hover].x}: <b>{points[hover].y}{unit}</b>
        </div>
      )}
    </div>
  );
}

/** Vertical single-series bars with hover. */
export function Bars({ data, unit = "", color = C_A, label }: { data: { label: string; value: number }[]; unit?: string; color?: string; label: string }) {
  const [hover, setHover] = useState<number | null>(null);
  const max = Math.max(...data.map((d) => d.value)) * 1.12;
  return (
    <div className="flex h-40 items-end gap-[2px]" role="img" aria-label={label}>
      {data.map((d, i) => (
        <div key={d.label} className="relative flex h-full flex-1 flex-col items-center justify-end" onMouseEnter={() => setHover(i)} onMouseLeave={() => setHover(null)}>
          {hover === i && <div className="absolute -top-1 z-10 whitespace-nowrap rounded-md bg-ink px-2 py-1 text-[11px] text-white">{d.label}: {d.value}{unit}</div>}
          <div className="w-full max-w-9 rounded-t-[4px] transition-all duration-700" style={{ height: `${(d.value / max) * 100}%`, background: color, opacity: hover === null || hover === i ? 1 : 0.4 }} />
          <div className="mt-1.5 w-full truncate text-center text-[10px] text-faint">{d.label}</div>
        </div>
      ))}
    </div>
  );
}
