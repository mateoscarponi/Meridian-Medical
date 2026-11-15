import Link from "next/link";
import type { ComponentProps, ReactNode } from "react";

export function cx(...c: (string | false | null | undefined)[]) {
  return c.filter(Boolean).join(" ");
}

export type Tone = "neutral" | "ocean" | "sage" | "gold" | "danger" | "warn" | "ink";

const TONES: Record<Tone, string> = {
  neutral: "bg-sand text-muted ring-line",
  ocean: "bg-sky-2 text-ocean ring-sky",
  sage: "bg-sage-soft text-sage ring-sage/20",
  gold: "bg-gold-soft text-gold ring-gold/20",
  danger: "bg-danger-soft text-danger ring-danger/20",
  warn: "bg-warn-soft text-warn ring-warn/20",
  ink: "bg-ink text-white ring-ink",
};

export function Badge({ tone = "neutral", children, className }: { tone?: Tone; children: ReactNode; className?: string }) {
  return (
    <span className={cx("inline-flex items-center gap-1 whitespace-nowrap rounded-full px-2 py-0.5 text-[11px] font-semibold ring-1 ring-inset", TONES[tone], className)}>
      {children}
    </span>
  );
}

export function Card({ className, children, ...rest }: ComponentProps<"div">) {
  return (
    <div className={cx("rounded-2xl border border-line bg-white shadow-[var(--shadow-soft)]", className)} {...rest}>
      {children}
    </div>
  );
}

export function CardHeader({ title, subtitle, right, icon }: { title: ReactNode; subtitle?: ReactNode; right?: ReactNode; icon?: ReactNode }) {
  return (
    <div className="flex items-start justify-between gap-3 border-b border-line px-5 py-4">
      <div className="flex min-w-0 items-start gap-3">
        {icon && <div className="mt-0.5 grid size-8 shrink-0 place-items-center rounded-lg bg-sky-2 text-ocean">{icon}</div>}
        <div className="min-w-0">
          <h3 className="text-[15px] font-semibold text-ink">{title}</h3>
          {subtitle && <p className="mt-0.5 text-[13px] text-muted">{subtitle}</p>}
        </div>
      </div>
      {right && <div className="shrink-0">{right}</div>}
    </div>
  );
}

type BtnVariant = "primary" | "secondary" | "ghost" | "sage" | "dark";
const BTN: Record<BtnVariant, string> = {
  primary: "bg-ocean text-white hover:bg-navy shadow-sm",
  sage: "bg-sage text-white hover:bg-[#25724f] shadow-sm",
  dark: "bg-ink text-white hover:bg-ink-2 shadow-sm",
  secondary: "bg-white text-ink ring-1 ring-inset ring-line-2 hover:bg-sand",
  ghost: "text-ocean hover:bg-sky-2",
};

export function Button({ variant = "primary", size = "md", className, ...rest }: ComponentProps<"button"> & { variant?: BtnVariant; size?: "sm" | "md" }) {
  return (
    <button
      className={cx(
        "inline-flex items-center justify-center gap-2 rounded-xl font-semibold transition-all duration-200 disabled:cursor-not-allowed disabled:opacity-50 active:scale-[0.98]",
        size === "sm" ? "px-3 py-1.5 text-[13px]" : "px-4 py-2.5 text-sm",
        BTN[variant],
        className,
      )}
      {...rest}
    />
  );
}

export function LinkButton({ href, variant = "primary", className, children }: { href: string; variant?: BtnVariant; className?: string; children: ReactNode }) {
  return (
    <Link href={href} className={cx("inline-flex items-center justify-center gap-2 rounded-xl px-5 py-3 text-sm font-semibold transition-all duration-200 active:scale-[0.98]", BTN[variant], className)}>
      {children}
    </Link>
  );
}

export function Stat({ label, value, delta, tone = "sage", hint }: { label: string; value: ReactNode; delta?: string; tone?: "sage" | "danger" | "neutral"; hint?: string }) {
  return (
    <Card className="p-4">
      <div className="text-[12px] font-medium uppercase tracking-wide text-faint">{label}</div>
      <div className="mt-1.5 flex items-baseline gap-2">
        <div className="whitespace-nowrap font-display text-[28px] leading-none font-semibold text-ink tabular-nums">{value}</div>
        {delta && <span className={cx("text-[12px] font-semibold", tone === "sage" ? "text-sage" : tone === "danger" ? "text-danger" : "text-muted")}>{delta}</span>}
      </div>
      {hint && <div className="mt-1.5 text-[12px] text-muted">{hint}</div>}
    </Card>
  );
}

export function Progress({ value, tone = "ocean", className }: { value: number; tone?: "ocean" | "sage" | "gold" | "danger" | "warn"; className?: string }) {
  const color = { ocean: "bg-ocean", sage: "bg-sage", gold: "bg-gold", danger: "bg-danger", warn: "bg-warn" }[tone];
  return (
    <div className={cx("h-1.5 w-full overflow-hidden rounded-full bg-sand", className)} role="progressbar" aria-valuenow={Math.round(value)} aria-valuemin={0} aria-valuemax={100}>
      <div className={cx("h-full rounded-full transition-[width] duration-700 ease-out", color)} style={{ width: `${Math.max(0, Math.min(100, value))}%` }} />
    </div>
  );
}

export function Eyebrow({ children, className }: { children: ReactNode; className?: string }) {
  return <div className={cx("text-[12px] font-semibold uppercase tracking-[0.16em] text-gold", className)}>{children}</div>;
}

export function CodeBlock({ value, className, maxH = "max-h-96" }: { value: unknown; className?: string; maxH?: string }) {
  const text = typeof value === "string" ? value : JSON.stringify(value, null, 2);
  return (
    <pre className={cx("scroll-thin overflow-auto rounded-xl bg-ink p-4 font-mono text-[12px] leading-relaxed text-[#cfe1ee]", maxH, className)}>
      <code>{text}</code>
    </pre>
  );
}

export function Segmented<T extends string>({ value, onChange, options }: { value: T; onChange: (v: T) => void; options: { value: T; label: string }[] }) {
  return (
    <div className="inline-flex rounded-xl bg-sand p-1 ring-1 ring-inset ring-line">
      {options.map((o) => (
        <button
          key={o.value}
          onClick={() => onChange(o.value)}
          className={cx(
            "rounded-lg px-3 py-1.5 text-[13px] font-semibold transition-all",
            value === o.value ? "bg-white text-ink shadow-sm" : "text-muted hover:text-ink",
          )}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}

export function Field({ label, children, hint }: { label: string; children: ReactNode; hint?: string }) {
  return (
    <label className="block">
      <span className="mb-1.5 block text-[12px] font-semibold text-muted">{label}</span>
      {children}
      {hint && <span className="mt-1 block text-[11px] text-faint">{hint}</span>}
    </label>
  );
}

export const inputCls =
  "w-full rounded-xl border border-line-2 bg-white px-3 py-2 text-sm text-ink outline-none transition focus:border-ocean focus:ring-4 focus:ring-sky";

export function money(n: number, digits = 0) {
  return n.toLocaleString("en-US", { style: "currency", currency: "USD", maximumFractionDigits: digits, minimumFractionDigits: digits });
}
