"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import { Menu, Phone, X, ArrowUpRight } from "lucide-react";
import { Logo, DemoPill } from "./brand";
import { PRACTICE } from "@/lib/practice";
import { cx } from "../ui";

const NAV = [
  { href: "/#specialties", label: "Specialties" },
  { href: "/#conditions", label: "Conditions" },
  { href: "/#team", label: "Our Team" },
  { href: "/#locations", label: "Locations" },
  { href: "/#patients", label: "For Patients" },
];

export function SiteHeader() {
  const [open, setOpen] = useState(false);
  const [scrolled, setScrolled] = useState(false);
  useEffect(() => {
    const on = () => setScrolled(window.scrollY > 8);
    on();
    window.addEventListener("scroll", on, { passive: true });
    return () => window.removeEventListener("scroll", on);
  }, []);

  return (
    <header className="sticky top-0 z-50">
      <div className="bg-ink text-[12px] text-white/75">
        <div className="mx-auto flex max-w-7xl items-center justify-between gap-4 px-4 py-2 sm:px-6">
          <div className="flex items-center gap-2.5">
            <DemoPill />
            <span className="hidden sm:inline">Serving {PRACTICE.region} since {PRACTICE.since}</span>
          </div>
          <div className="flex items-center gap-4">
            <Link href="/#locations" className="hidden hover:text-white md:inline">Hicksville Walk-in Injury Clinic · No appointment needed</Link>
            <a href="tel:5165550148" className="inline-flex items-center gap-1.5 font-semibold text-white">
              <Phone className="size-3.5" /> {PRACTICE.phone}
            </a>
          </div>
        </div>
      </div>
      <div className={cx("border-b transition-all duration-300", scrolled ? "border-line bg-cream/85 shadow-[0_8px_30px_-20px_rgb(15_30_46/0.35)] backdrop-blur-xl" : "border-transparent bg-cream")}>
        <div className="mx-auto flex max-w-7xl items-center justify-between gap-6 px-4 py-3.5 sm:px-6">
          <Logo />
          <nav className="hidden items-center gap-1 whitespace-nowrap xl:flex" aria-label="Primary">
            {NAV.map((n) => (
              <Link key={n.href} href={n.href} className="rounded-lg px-3 py-2 text-[14px] font-medium text-muted transition-colors hover:bg-sand hover:text-ink">
                {n.label}
              </Link>
            ))}
            <Link href="/platform" className="ml-1 inline-flex items-center gap-1 rounded-lg px-3 py-2 text-[14px] font-semibold text-ocean transition-colors hover:bg-sky-2">
              Care Platform <ArrowUpRight className="size-3.5" />
            </Link>
          </nav>
          <div className="hidden items-center gap-2 whitespace-nowrap xl:flex">
            <Link href="/#request" className="rounded-xl bg-navy px-4 py-2.5 text-[14px] font-semibold text-white shadow-sm transition-all hover:bg-ocean active:scale-[0.98]">
              Request appointment
            </Link>
          </div>
          <button className="rounded-lg p-2 text-ink xl:hidden" onClick={() => setOpen((v) => !v)} aria-label={open ? "Close menu" : "Open menu"} aria-expanded={open}>
            {open ? <X className="size-6" /> : <Menu className="size-6" />}
          </button>
        </div>
        {open && (
          <div className="border-t border-line bg-cream px-4 pb-5 pt-2 xl:hidden">
            {NAV.map((n) => (
              <Link key={n.href} href={n.href} onClick={() => setOpen(false)} className="block rounded-lg px-3 py-3 text-[15px] font-medium text-ink hover:bg-sand">
                {n.label}
              </Link>
            ))}
            <Link href="/platform" className="block rounded-lg px-3 py-3 text-[15px] font-semibold text-ocean hover:bg-sky-2">
              Care Platform →
            </Link>
            <Link href="/#request" onClick={() => setOpen(false)} className="mt-2 block rounded-xl bg-navy px-4 py-3 text-center text-[15px] font-semibold text-white">
              Request appointment
            </Link>
          </div>
        )}
      </div>
    </header>
  );
}
