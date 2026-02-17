"use client";

import { useState } from "react";
import { docsNav } from "./nav";

export default function Sidebar({ currentSlug }: { currentSlug?: string }) {
  const [open, setOpen] = useState(false);

  return (
    <>
      {/* Mobile toggle */}
      <button
        onClick={() => setOpen(!open)}
        className="fixed top-20 left-4 z-20 rounded-lg border border-border bg-background p-2 shadow-sm md:hidden"
        aria-label="Toggle docs navigation"
      >
        <svg
          className="h-5 w-5"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          {open ? (
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M6 18L18 6M6 6l12 12"
            />
          ) : (
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M4 6h16M4 12h16M4 18h16"
            />
          )}
        </svg>
      </button>

      {/* Backdrop */}
      {open && (
        <div
          className="fixed inset-0 z-10 bg-black/20 md:hidden"
          onClick={() => setOpen(false)}
        />
      )}

      {/* Sidebar */}
      <aside
        className={`fixed top-16 left-0 z-10 h-[calc(100vh-4rem)] w-64 shrink-0 overflow-y-auto border-r border-border bg-background p-6 transition-transform md:sticky md:translate-x-0 ${
          open ? "translate-x-0" : "-translate-x-full"
        }`}
      >
        <nav>
          <h3 className="mb-3 text-xs font-semibold uppercase tracking-wider text-muted">
            Documentation
          </h3>
          <ul className="space-y-1">
            {docsNav.map((item) => (
              <li key={item.slug}>
                <a
                  href={`/docs/${item.slug}`}
                  onClick={() => setOpen(false)}
                  className={`block rounded-md px-3 py-2 text-sm transition-colors ${
                    currentSlug === item.slug
                      ? "bg-accent/10 font-medium text-accent"
                      : "text-muted hover:bg-code-bg hover:text-foreground"
                  }`}
                >
                  {item.title}
                </a>
              </li>
            ))}
          </ul>
        </nav>
      </aside>
    </>
  );
}
