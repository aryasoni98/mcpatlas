"use client";

import { motion } from "framer-motion";
import { Section } from "@/components/ui/Section";
import { Heading, Text } from "@/components/ui/Typography";
import { useReducedMotion } from "@/components/motion";
import { sectionReveal, viewportOnce } from "@/components/motion/sectionVariants";
import { STACK_LIST } from "@/lib/constants";

export function Solution() {
  const reduced = useReducedMotion();

  return (
    <Section id="solution">
      <div className="grid gap-12 md:grid-cols-[1fr_auto] md:items-start">
        <motion.div
          variants={reduced ? undefined : sectionReveal}
          initial="hidden"
          whileInView="visible"
          viewport={viewportOnce}
          className="max-w-[40rem]"
        >
          <Heading as="h2">One server, full landscape</Heading>
          <Text className="mt-4 text-base leading-[1.6]" muted>
            MCPAtlas exposes the entire CNCF landscape over the Model Context
            Protocol — 15 tools including issue context for AI-assisted
            contribution, project search, comparison, health scores, stack
            recommendations, migration guides, and a knowledge graph. Use it
            from Claude, Cursor, VS Code, or any MCP client — STDIO, SSE, or
            Streamable HTTP.
          </Text>
        </motion.div>
        <motion.div
          variants={reduced ? undefined : sectionReveal}
          initial="hidden"
          whileInView="visible"
          viewport={viewportOnce}
          transition={{ delay: 0.1 }}
          className="rounded-xl border border-border bg-bg-elevated px-5 py-4 font-mono text-[0.8125rem] leading-[1.5] text-muted-foreground shadow-md"
        >
          {STACK_LIST}
        </motion.div>
      </div>
    </Section>
  );
}
