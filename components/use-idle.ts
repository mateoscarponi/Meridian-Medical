"use client";

import { useEffect, useRef, useState } from "react";

/**
 * Automatic logoff (HIPAA Security Rule §164.312(a)(2)(iii)).
 * Returns true once the user has been inactive for `ms`; `reset()` clears it.
 */
export function useIdle(ms: number) {
  const [idle, setIdle] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    const arm = () => {
      if (timer.current) clearTimeout(timer.current);
      timer.current = setTimeout(() => setIdle(true), ms);
    };
    const events = ["mousemove", "keydown", "pointerdown", "scroll", "touchstart"] as const;
    events.forEach((e) => window.addEventListener(e, arm, { passive: true }));
    arm();
    return () => {
      events.forEach((e) => window.removeEventListener(e, arm));
      if (timer.current) clearTimeout(timer.current);
    };
  }, [ms]);

  return { idle, reset: () => setIdle(false) };
}
