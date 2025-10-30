"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";

export function ThemeToggle() {
  const [isDark, setIsDark] = useState<boolean>(() => {
    if (typeof window === "undefined") return false;
    const stored = localStorage.getItem("theme");
    const prefersDark = window.matchMedia?.("(prefers-color-scheme: dark)")?.matches;
    const shouldDark = (stored ?? (prefersDark ? "dark" : "light")) === "dark";
    document.documentElement.classList.toggle("dark", shouldDark);
    return shouldDark;
  });

  const toggle = () => {
    const next = !isDark;
    setIsDark(next);
    document.documentElement.classList.toggle("dark", next);
    localStorage.setItem("theme", next ? "dark" : "light");
  };

  return (
    <Button variant="outline" size="md" aria-label="Toggle theme" onClick={toggle}>
      {isDark ? "Light" : "Dark"}
    </Button>
  );
}
