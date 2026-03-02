"use client";

import {
  BookOpen,
  Database,
  GitBranch,
  LayoutGrid,
  Radio,
  Search,
} from "lucide-react";
import { motion } from "framer-motion";
import { FEATURES } from "@/lib/constants";
import { Section } from "@/components/ui/Section";
import { Heading } from "@/components/ui/Typography";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import {
  staggerContainer,
  staggerItem,
  reducedStaggerContainer,
  reducedStaggerItem,
  viewportOnce,
} from "@/components/motion";
import { useReducedMotion } from "@/components/motion";

const FEATURE_ICONS = [
  LayoutGrid,
  BookOpen,
  Radio,
  GitBranch,
  Search,
  Database,
] as const;

/** Topic-specific icon animation variants (respects reduced motion via no transition). */
const iconVariants = {
  tools: {
    hidden: { opacity: 0, scale: 0.8 },
    visible: { opacity: 1, scale: 1, transition: { duration: 0.35 } },
    hover: { scale: 1.05 },
  },
  resources: {
    hidden: { opacity: 0, y: 4 },
    visible: { opacity: 1, y: 0, transition: { duration: 0.35 } },
    hover: { scale: 1.05 },
  },
  transport: {
    hidden: { opacity: 0, x: -8 },
    visible: { opacity: 1, x: 0, transition: { duration: 0.35 } },
    hover: { scale: 1.05 },
  },
  graph: {
    hidden: { opacity: 0 },
    visible: { opacity: 1, transition: { duration: 0.4 } },
    hover: { scale: 1.05 },
  },
  search: {
    hidden: { opacity: 0, scale: 0.9 },
    visible: { opacity: 1, scale: 1, transition: { duration: 0.35 } },
    hover: { scale: 1.08 },
  },
  projects: {
    hidden: { opacity: 0, y: 6 },
    visible: { opacity: 1, y: 0, transition: { duration: 0.35 } },
    hover: { y: -2 },
  },
} as const;

const ICON_ANIMATION_KEYS = [
  "tools",
  "resources",
  "transport",
  "graph",
  "search",
  "projects",
] as const;

/** Mini animated graph for Knowledge graph card: nodes + edges. */
function AnimatedGraph() {
  const reduced = useReducedMotion();
  const lineTransition = { duration: 0.4 };
  const nodePositions = [
    { cx: 16, cy: 24, delay: 0 },
    { cx: 40, cy: 24, delay: 0.1 },
    { cx: 64, cy: 24, delay: 0.2 },
    { cx: 40, cy: 8, delay: 0.15 },
    { cx: 40, cy: 40, delay: 0.25 },
  ];
  return (
    <div className="mt-3 flex h-14 items-center justify-center rounded-lg border border-zinc-700/80 bg-zinc-800/40 p-2">
      <svg
        viewBox="0 0 80 48"
        className="h-full w-full max-w-[120px]"
        aria-hidden
      >
        {/* Edges — animate opacity */}
        <motion.line
          x1="16"
          y1="24"
          x2="40"
          y2="24"
          stroke="currentColor"
          strokeWidth="1.5"
          className="text-zinc-500"
          initial={reduced ? undefined : { opacity: 0 }}
          animate={
            reduced ? undefined : { opacity: 0.7, transition: lineTransition }
          }
        />
        <motion.line
          x1="40"
          y1="24"
          x2="64"
          y2="24"
          stroke="currentColor"
          strokeWidth="1.5"
          className="text-zinc-500"
          initial={reduced ? undefined : { opacity: 0 }}
          animate={
            reduced
              ? undefined
              : { opacity: 0.7, transition: { ...lineTransition, delay: 0.08 } }
          }
        />
        <motion.line
          x1="40"
          y1="24"
          x2="40"
          y2="8"
          stroke="currentColor"
          strokeWidth="1.5"
          className="text-zinc-500"
          initial={reduced ? undefined : { opacity: 0 }}
          animate={
            reduced
              ? undefined
              : { opacity: 0.7, transition: { ...lineTransition, delay: 0.12 } }
          }
        />
        <motion.line
          x1="40"
          y1="24"
          x2="40"
          y2="40"
          stroke="currentColor"
          strokeWidth="1.5"
          className="text-zinc-500"
          initial={reduced ? undefined : { opacity: 0 }}
          animate={
            reduced
              ? undefined
              : { opacity: 0.7, transition: { ...lineTransition, delay: 0.16 } }
          }
        />
        {/* Nodes */}
        {nodePositions.map(({ cx, cy, delay }, i) => (
          <motion.circle
            key={i}
            r="4"
            cx={cx}
            cy={cy}
            fill="currentColor"
            className="text-zinc-400"
            initial={reduced ? undefined : { scale: 0, opacity: 0 }}
            animate={
              reduced
                ? undefined
                : {
                    scale: 1,
                    opacity: 1,
                    transition: { duration: 0.25, delay: 0.15 + delay },
                  }
            }
          />
        ))}
      </svg>
    </div>
  );
}

export function FeaturesGrid() {
  const reduced = useReducedMotion();
  const containerVariants = reduced ? reducedStaggerContainer : staggerContainer;
  const itemVariants = reduced ? reducedStaggerItem : staggerItem;

  return (
    <Section
      id="features"
      className="bg-zinc-950 dark:bg-black"
    >
      <Heading as="h2" className="mb-10 text-white">
        Features
      </Heading>
      <motion.div
        className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3"
        variants={containerVariants}
        initial="hidden"
        whileInView="visible"
        viewport={viewportOnce}
      >
        {FEATURES.map((item, i) => {
          const Icon = FEATURE_ICONS[i];
          const iconKey = ICON_ANIMATION_KEYS[i];
          const iconMotion = iconVariants[iconKey];
          const isKnowledgeGraph = item.title === "Knowledge graph";

          return (
            <motion.div key={item.title} variants={itemVariants}>
              <motion.div
                whileHover={reduced ? undefined : { y: -2 }}
                transition={{ duration: 0.2 }}
                className="h-full"
              >
                <Card className="h-full rounded-xl border-zinc-700/80 bg-zinc-900/80 shadow-black/5 transition-shadow dark:border-zinc-800 dark:bg-zinc-900/50 hover:border-zinc-600/80 dark:hover:border-zinc-600/50">
                  <CardHeader className="space-y-0 pb-2">
                    <motion.div
                      className="flex h-9 w-9 items-center justify-center rounded-lg border border-zinc-700 bg-zinc-800/80 dark:border-zinc-700 dark:bg-zinc-800/50"
                      variants={iconMotion}
                      initial="hidden"
                      animate="visible"
                      whileHover={reduced ? undefined : iconMotion.hover}
                      transition={{
                        duration: 0.3,
                        delay: 0.12 + i * 0.06,
                      }}
                    >
                      <Icon className="h-4 w-4 text-white" />
                    </motion.div>
                    <h3 className="mt-3 text-lg font-semibold leading-tight text-white">
                      {item.title}
                    </h3>
                  </CardHeader>
                  <CardContent className="pt-0">
                    {isKnowledgeGraph && <AnimatedGraph />}
                    <p className="text-sm leading-[1.5] text-zinc-400 dark:text-zinc-400">
                      {item.description}
                    </p>
                    {item.mono ? (
                      <p
                        className="mt-2 font-mono text-[0.75rem] leading-[1.4] text-zinc-500"
                        aria-hidden
                      >
                        {item.mono}
                      </p>
                    ) : null}
                  </CardContent>
                </Card>
              </motion.div>
            </motion.div>
          );
        })}
      </motion.div>
    </Section>
  );
}
