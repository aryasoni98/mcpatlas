"use client";

import { motion } from "framer-motion";
import { useReducedMotion } from "@/components/motion";
import { useEffect, useState } from "react";

/** Terminal lines per spec: real commands and tool names; accent on key tokens. */
const LINES: { text: string; accent?: string }[] = [
  { text: "$ cncf-mcp --transport stdio" },
  { text: "> tools/list" },
  {
    text: "  search_projects, get_project, compare_projects, ...",
    accent: "search_projects, get_project, compare_projects",
  },
  {
    text: "> tools/call search_projects {\"query\": \"service mesh\"}",
    accent: "search_projects",
  },
  {
    text: "  ✓ 12 results · 2,400+ projects indexed",
    accent: "✓",
  },
];

const LINE_DELAY_MS = 100;

/** Microterminal: dark surface, mono, line-by-line reveal (80–120ms), blinking cursor. */
export function TerminalBlock({ className = "" }: { className?: string }) {
  const reduced = useReducedMotion();
  const [visibleCount, setVisibleCount] = useState(0);

  useEffect(() => {
    if (reduced) {
      setVisibleCount(LINES.length);
      return;
    }
    const id = setInterval(() => {
      setVisibleCount((n) => (n < LINES.length ? n + 1 : n));
    }, LINE_DELAY_MS);
    return () => clearInterval(id);
  }, [reduced]);

  const showAll = reduced || visibleCount >= LINES.length;

  return (
    <div
      className={`rounded-xl border border-border bg-bg-elevated px-4 py-3 font-mono text-[0.8125rem] shadow-md ${className}`}
      aria-hidden
    >
      <div className="mb-2 flex items-center gap-2">
        <span className="h-2 w-2 rounded-full bg-accent-warning/80" />
        <span className="h-2 w-2 rounded-full bg-amber-500/80" />
        <span className="h-2 w-2 rounded-full bg-accent-success/80" />
      </div>
      <div className="space-y-1 text-muted-foreground">
        {LINES.map((line, i) => (
          <motion.div
            key={i}
            initial={reduced ? false : { opacity: 0 }}
            animate={{
              opacity: showAll || i < visibleCount ? 1 : 0,
            }}
            transition={{ duration: 0.2 }}
            className="flex items-center gap-1"
          >
            {i === visibleCount && !showAll ? (
              <span
                className="inline-block h-4 w-2 bg-accent-brand animate-cursor-blink"
                aria-hidden
              />
            ) : null}
            {line.accent ? (
              <span>
                {line.text.split(line.accent)[0]}
                <span className="text-accent-brand">
                  {line.accent}
                </span>
                {line.text.split(line.accent)[1]}
              </span>
            ) : (
              <span>{line.text}</span>
            )}
          </motion.div>
        ))}
      </div>
      {showAll && (
        <span
          className="mt-1 inline-block h-4 w-2 bg-[hsl(var(--accent-brand))] animate-cursor-blink"
          aria-hidden
        />
      )}
    </div>
  );
}
