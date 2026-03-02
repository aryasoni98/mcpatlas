"use client";

import { motion, useScroll, useTransform } from "framer-motion";
import { Button } from "@/components/ui/button";
import { KnowledgeGraphMesh } from "@/components/diagram";
import { MCPFlowDiagram } from "@/components/landing/MCPFlowDiagram";
import { TerminalBlock } from "@/components/landing/TerminalBlock";
import { useReducedMotion } from "@/components/motion";
import { DOCS_URL, GITHUB_URL } from "@/lib/constants";

const easeOutExpo = [0.22, 1, 0.36, 1] as const;

export function Hero() {
  const reduced = useReducedMotion();
  const { scrollY } = useScroll();
  const backgroundY = useTransform(scrollY, [0, 400], [0, 80]);
  const heroContentOpacity = useTransform(scrollY, [0, 200], [1, 0.7]);
  const heroContentScale = useTransform(scrollY, [0, 200], [1, 0.98]);

  return (
    <section className="relative flex min-h-[100dvh] w-full flex-col items-center justify-center overflow-hidden bg-bg-base pt-16 md:flex-row md:pt-0">
      {/* Layer 0: mesh background + radial accent fade (parallax) */}
      <div className="pointer-events-none absolute inset-0">
        <div
          className="absolute inset-0"
          style={{
            background:
              "radial-gradient(ellipse 80% 50% at 50% 0%, var(--accent-glow), transparent 50%)",
          }}
        />
        <motion.div
          className="absolute inset-0"
          style={reduced ? undefined : { y: backgroundY }}
        >
          <KnowledgeGraphMesh />
        </motion.div>
      </div>

      <motion.div
        className="relative z-10 mx-auto flex w-full max-w-[72rem] flex-col gap-12 px-6 py-16 md:flex-row md:items-center md:gap-16 md:px-8"
        style={
          reduced
            ? undefined
            : {
                opacity: heroContentOpacity,
                scale: heroContentScale,
              }
        }
      >
        {/* Left: headline, subline, CTAs */}
        <div className="flex max-w-[540px] flex-col text-left">
          <motion.h1
            className="text-[2.75rem] font-semibold leading-[1.1] tracking-tight text-foreground md:text-[3rem] lg:text-[3.5rem]"
            initial={reduced ? false : { opacity: 0, y: 12 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{
              duration: 0.5,
              delay: 0.1,
              ease: easeOutExpo,
            }}
          >
            The CNCF Landscape, in your AI assistant.
          </motion.h1>
          <motion.p
            className="mt-6 text-lg leading-relaxed text-muted-foreground"
            initial={reduced ? false : { opacity: 0, y: 12 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{
              duration: 0.4,
              delay: 0.35,
              ease: easeOutExpo,
            }}
          >
            One server. 2,400+ projects. MCP tools, resources, prompts. STDIO,
            SSE, or Streamable HTTP.
          </motion.p>
          <motion.div
            className="mt-10 flex flex-wrap gap-4"
            initial={reduced ? false : { opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{
              duration: 0.35,
              delay: 0.5,
              ease: easeOutExpo,
            }}
          >
            <Button
              asChild
              size="lg"
              className="bg-accent-brand text-white hover:opacity-90 focus-visible:outline-ring"
            >
              <a href={DOCS_URL}>Get started</a>
            </Button>
            <Button
              asChild
              variant="outline"
              size="lg"
              className="border-border"
            >
              <a href={GITHUB_URL}>GitHub</a>
            </Button>
          </motion.div>
        </div>

        {/* Right: MCP flow diagram + terminal */}
        <motion.div
          className="flex w-full flex-col gap-6 md:max-w-[420px] md:flex-shrink-0"
          initial={reduced ? false : { opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{
            duration: 0.4,
            delay: 0.6,
            ease: easeOutExpo,
          }}
        >
          <MCPFlowDiagram className="w-full" />
          <TerminalBlock />
        </motion.div>
      </motion.div>
    </section>
  );
}
