"use client";

import dynamic from "next/dynamic";

// WebGL only runs in the browser; load the 3D scene client-side, with a soft placeholder meanwhile.
const Spine3D = dynamic(() => import("./spine-3d"), {
  ssr: false,
  loading: () => <div className="h-full w-full animate-pulse-soft rounded-3xl bg-gradient-to-r from-transparent via-sky/40 to-transparent" />,
});

export function HeroSpine() {
  return (
    <div className="relative mt-4 h-56 sm:h-64 lg:h-[270px]">
      {/* Soft glow and floor shadow so the model sits on the page instead of floating on a hole */}
      <div className="pointer-events-none absolute inset-x-10 top-1/2 h-32 -translate-y-1/2 rounded-full bg-gradient-to-r from-sky/60 via-gold-soft/70 to-sky/50 blur-3xl" />
      <div className="pointer-events-none absolute inset-x-20 bottom-3 h-6 rounded-[50%] bg-ink/10 blur-xl" />
      <Spine3D className="relative h-full w-full" />
    </div>
  );
}
