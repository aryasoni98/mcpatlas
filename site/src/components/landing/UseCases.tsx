"use client";

import { motion } from "framer-motion";
import { USE_CASES } from "@/lib/constants";
import { Section } from "@/components/ui/Section";
import { Heading } from "@/components/ui/Typography";
import {
  staggerContainer,
  staggerItem,
  reducedStaggerContainer,
  reducedStaggerItem,
  viewportOnce,
} from "@/components/motion";
import { useReducedMotion } from "@/components/motion";

export function UseCases() {
  const reduced = useReducedMotion();
  const containerVariants = reduced ? reducedStaggerContainer : staggerContainer;
  const itemVariants = reduced ? reducedStaggerItem : staggerItem;

  return (
    <Section id="use-cases">
      <Heading as="h2">Use cases</Heading>
      <motion.ul
        className="mt-10 grid gap-4 sm:grid-cols-2"
        variants={containerVariants}
        initial="hidden"
        whileInView="visible"
        viewport={viewportOnce}
      >
        {USE_CASES.map((item) => (
          <motion.li
            key={item.title}
            variants={itemVariants}
            className="rounded-xl border border-border bg-bg-elevated p-5 shadow-sm"
          >
            <h3 className="font-semibold text-foreground">
              {item.title}
            </h3>
            <p className="mt-1 text-sm text-muted-foreground">
              {item.description}
            </p>
            <p
              className="mt-2 font-mono text-[0.75rem] leading-[1.4] text-muted-foreground"
              aria-hidden
            >
              {item.tools}
            </p>
          </motion.li>
        ))}
      </motion.ul>
    </Section>
  );
}
