"use client";

import { ReactNode, useEffect } from "react";

type Props = { children: ReactNode };

export function Providers({ children }: Props) {
  // Apply stored theme on mount without state updates.
  useEffect(() => {
    const stored = typeof window !== "undefined" ? localStorage.getItem("theme") : null;
    const prefersDark = window.matchMedia?.("(prefers-color-scheme: dark)")?.matches;
    const theme = stored ?? (prefersDark ? "dark" : "light");
    document.documentElement.classList.toggle("dark", theme === "dark");
  }, []);

  return <>{children}</>;
}
