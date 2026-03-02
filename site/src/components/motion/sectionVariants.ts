/**
 * Shared motion variants for landing sections (2026 AI-native redesign).
 * Use with Framer Motion; respects prefers-reduced-motion when useReducedMotion() is true.
 */

import type { Variants } from "framer-motion";

const easeOutExpo = [0.22, 1, 0.36, 1] as const;
const durationNormal = 0.4;
const durationSlow = 0.5;

/** Section reveal: fade + slight y. Use on section wrappers. */
export const sectionReveal: Variants = {
  hidden: { opacity: 0, y: 24 },
  visible: {
    opacity: 1,
    y: 0,
    transition: {
      duration: durationSlow,
      ease: easeOutExpo,
    },
  },
};

/** Stagger container: use with staggerChildren on parent. */
export const staggerContainer: Variants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.08,
      delayChildren: 0.1,
    },
  },
};

/** Stagger item: fade + y. Use on children of staggerContainer. */
export const staggerItem: Variants = {
  hidden: { opacity: 0, y: 16 },
  visible: {
    opacity: 1,
    y: 0,
    transition: {
      duration: durationNormal,
      ease: easeOutExpo,
    },
  },
};

/** No stagger: for reduced-motion. */
export const reducedStaggerContainer: Variants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: { staggerChildren: 0, delayChildren: 0 },
  },
};

/** No motion: for reduced-motion stagger children. */
export const reducedStaggerItem: Variants = {
  hidden: { opacity: 0 },
  visible: { opacity: 1, transition: { duration: 0.25 } },
};

/** Viewport options for once-only reveal. */
export const viewportOnce = { once: true, margin: "-50px" } as const;

/** Transition for hero-style entrances (slightly slower). */
export const heroTransition = {
  duration: 0.5,
  ease: easeOutExpo,
};
