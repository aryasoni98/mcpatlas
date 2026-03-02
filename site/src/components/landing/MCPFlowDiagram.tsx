"use client";

import { motion } from "framer-motion";
import { useReducedMotion } from "@/components/motion";

const NODES = [
  { id: "client", label: "Client" },
  { id: "transport", label: "Transport" },
  { id: "core", label: "Core" },
  { id: "tools", label: "Tools" },
  { id: "storage", label: "Storage" },
] as const;

/** Horizontal flow: Client → Transport → Core → Tools → Storage. SVG with animated edges. */
export function MCPFlowDiagram({ className = "" }: { className?: string }) {
  const reduced = useReducedMotion();

  const nodeDuration = reduced ? 0.2 : 0.35;
  const stagger = 0.08;

  return (
    <div className={className} aria-hidden>
      <svg
        viewBox="0 0 440 80"
        className="w-full max-w-full h-auto"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        {/* Edges: simple lines between node centers (approx 88px apart) */}
        <defs>
          <linearGradient id="flowGradient" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="var(--foreground-low)" />
            <stop offset="100%" stopColor="var(--primary-low)" />
          </linearGradient>
        </defs>
        {[0, 1, 2, 3].map((i) => {
          const x1 = 44 + i * 88;
          const x2 = 44 + (i + 1) * 88;
          const d = `M ${x1} 40 L ${x2} 40`;
          return (
            <motion.path
              key={`edge-${i}`}
              d={d}
              stroke="url(#flowGradient)"
              strokeWidth={1.5}
              strokeLinecap="round"
              initial={{ pathLength: 0 }}
              animate={{ pathLength: 1 }}
              transition={{
                duration: reduced ? 0 : 0.4,
                delay: reduced ? 0 : 0.6 + i * 0.08,
                ease: [0.22, 1, 0.36, 1],
              }}
            />
          );
        })}
        {/* Nodes: bg-elevated, border; appear with stagger */}
        {NODES.map((node, i) => (
          <g key={node.id}>
            <motion.rect
              x={8 + i * 88}
              y={12}
              width={72}
              height={56}
              rx={8}
              className="fill-bg-elevated stroke-border"
              strokeWidth={1}
              initial={{ opacity: 0, scale: 0.96 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{
                duration: nodeDuration,
                delay: reduced ? 0 : 0.6 + i * stagger,
                ease: [0.22, 1, 0.36, 1],
              }}
            />
            <motion.text
              x={44 + i * 88}
              y={44}
              textAnchor="middle"
              className="fill-foreground text-sm font-medium"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{
                duration: 0.2,
                delay: reduced ? 0 : 0.6 + i * stagger + 0.05,
              }}
            >
              {node.label}
            </motion.text>
          </g>
        ))}
      </svg>
    </div>
  );
}
