"use client";

import {
  BookOpen,
  FileSearch,
  Globe,
  LayoutGrid,
  Radio,
} from "lucide-react";

import { BentoCard, BentoGrid } from "@/components/ui/bento-grid";
import { DOCS_URL, GITHUB_URL } from "@/lib/constants";
import { Section } from "@/components/ui/Section";
import { Heading } from "@/components/ui/Typography";

const BENTO_FEATURES = [
  {
    Icon: LayoutGrid,
    name: "15 MCP tools",
    description:
      "search_projects, get_project, compare_projects, suggest_stack, get_relationships, get_migration_path, and more.",
    href: DOCS_URL,
    cta: "View docs",
    background: (
      <img
        src="https://images.unsplash.com/photo-1558494949-ef010cbdcc31?w=800&q=80"
        alt=""
        className="absolute -right-20 -top-20 h-48 w-48 opacity-60 object-cover"
      />
    ),
    className: "lg:row-start-1 lg:row-end-4 lg:col-start-2 lg:col-end-3",
  },
  {
    Icon: FileSearch,
    name: "Full-text & hybrid search",
    description:
      "Search across 2,400+ CNCF projects. Tantivy full-text with optional Qdrant vector search and RRF merge.",
    href: DOCS_URL,
    cta: "Learn more",
    background: (
      <img
        src="https://images.unsplash.com/photo-1517694712202-14dd9538aa97?w=800&q=80"
        alt=""
        className="absolute -right-20 -top-20 h-48 w-48 opacity-60 object-cover"
      />
    ),
    className: "lg:col-start-1 lg:col-end-2 lg:row-start-1 lg:row-end-3",
  },
  {
    Icon: Globe,
    name: "Resources & prompts",
    description:
      "cncf:// URIs and prompts: evaluate_tool, plan_migration, review_stack, onboard_contributor.",
    href: DOCS_URL,
    cta: "Explore resources",
    background: (
      <img
        src="https://images.unsplash.com/photo-1451187580459-43490279c0fa?w=800&q=80"
        alt=""
        className="absolute -right-20 -top-20 h-48 w-48 opacity-60 object-cover"
      />
    ),
    className: "lg:col-start-1 lg:col-end-2 lg:row-start-3 lg:row-end-4",
  },
  {
    Icon: Radio,
    name: "STDIO, SSE, HTTP",
    description:
      "Content-Length framing, SSE, and Streamable HTTP (MCP 2025-03-26) with session support.",
    href: DOCS_URL,
    cta: "See transports",
    background: (
      <img
        src="https://images.unsplash.com/photo-1504639725590-34d0984388bd?w=800&q=80"
        alt=""
        className="absolute -right-20 -top-20 h-48 w-48 opacity-60 object-cover"
      />
    ),
    className: "lg:col-start-3 lg:col-end-4 lg:row-start-1 lg:row-end-2",
  },
  {
    Icon: BookOpen,
    name: "2,400+ projects",
    description:
      "CNCF Landscape data with maturity, categories, GitHub enrichment, and optional Artifact Hub.",
    href: GITHUB_URL,
    cta: "View GitHub",
    background: (
      <img
        src="https://images.unsplash.com/photo-1461749280684-dccba630e2f6?w=800&q=80"
        alt=""
        className="absolute -right-20 -top-20 h-48 w-48 opacity-60 object-cover"
      />
    ),
    className: "lg:col-start-3 lg:col-end-4 lg:row-start-2 lg:row-end-4",
  },
];

export function BentoDemo() {
  return (
    <Section id="bento">
      <Heading as="h2" className="mb-10">
        Explore by capability
      </Heading>
      <BentoGrid className="lg:grid-rows-3">
        {BENTO_FEATURES.map((feature) => (
          <BentoCard key={feature.name} {...feature} />
        ))}
      </BentoGrid>
    </Section>
  );
}
