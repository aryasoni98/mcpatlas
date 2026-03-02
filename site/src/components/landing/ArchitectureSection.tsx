"use client";

import { motion } from "framer-motion";
import { Section } from "@/components/ui/Section";
import { Heading, Text } from "@/components/ui/Typography";
import { MCPFlowDiagram } from "@/components/landing/MCPFlowDiagram";
import { useReducedMotion } from "@/components/motion";
import { sectionReveal, viewportOnce } from "@/components/motion/sectionVariants";

const PIPELINE_LINE =
  "Landscape YAML → Transport (STDIO | SSE | HTTP) → Core (tools, resources, prompts) → Storage (search, graph, vector, cache)";

export function ArchitectureSection() {
  const reduced = useReducedMotion();

  return (
    <Section id="architecture">
      <motion.div
        variants={reduced ? undefined : sectionReveal}
        initial="hidden"
        whileInView="visible"
        viewport={viewportOnce}
      >
        <Heading as="h2">Architecture</Heading>
        <Text className="mt-4 max-w-3xl" muted>
          Clients (Claude, Cursor, custom agents) talk to the MCP server over
          STDIO, SSE, or Streamable HTTP. The core routes requests to tools,
          resources, and prompts; a trait-based storage layer handles search
          (Tantivy), graph (in-memory or SurrealDB), vector (Qdrant), and cache.
          Data is piped from the CNCF Landscape YAML, GitHub, Artifact Hub, and
          optional embeddings.
        </Text>
        <div className="mt-8 flex justify-center">
          <MCPFlowDiagram className="mx-auto max-w-md" />
        </div>
        <div className="mt-6 overflow-x-auto rounded-xl border border-border bg-muted/50 px-4 py-3">
          <pre className="font-mono text-[0.75rem] leading-[1.4] text-foreground md:text-sm">
            {PIPELINE_LINE}
          </pre>
        </div>
      </motion.div>
    </Section>
  );
}
