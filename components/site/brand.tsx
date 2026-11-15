import Link from "next/link";
import { cx } from "../ui";

export function Monogram({ className, light = false }: { className?: string; light?: boolean }) {
  return (
    <span
      className={cx(
        "grid place-items-center rounded-xl font-display font-semibold tracking-tight",
        light ? "bg-white/10 text-white ring-1 ring-white/20" : "bg-navy text-white",
        className,
      )}
      aria-hidden
    >
      MO
    </span>
  );
}

export function Logo({ light = false, compact = false }: { light?: boolean; compact?: boolean }) {
  return (
    <Link href="/" className="group flex items-center gap-3" aria-label="Mateo Orthopedic & Spine Care, home">
      <Monogram light={light} className="size-10 text-[17px] transition-transform duration-300 group-hover:-rotate-3" />
      {!compact && (
        <span className="leading-tight">
          <span className={cx("block font-display text-[17px] font-semibold", light ? "text-white" : "text-ink")}>Mateo Orthopedic</span>
          <span className={cx("block text-[11px] font-semibold uppercase tracking-[0.18em]", light ? "text-white/60" : "text-muted")}>&amp; Spine Care</span>
        </span>
      )}
    </Link>
  );
}

export function DemoPill({ className }: { className?: string }) {
  return (
    <span
      className={cx("inline-flex items-center gap-1 rounded-full bg-gold/15 px-2 py-0.5 text-[10px] font-bold uppercase tracking-[0.14em] text-gold-2 ring-1 ring-inset ring-gold/30", className)}
      title="Concept demo — not a real medical practice. All data is synthetic."
    >
      Demo
    </span>
  );
}
