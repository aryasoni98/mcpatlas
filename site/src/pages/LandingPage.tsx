import { Suspense, lazy } from "react";
import { FloatingIconsHero } from "@/components/ui/floating-icons-hero-section";
import { heroFloatingIcons } from "@/components/landing/heroIcons";
import { DOCS_URL, GITHUB_URL } from "@/lib/constants";

const Problem = lazy(() =>
  import("@/components/landing/Problem").then((m) => ({ default: m.Problem }))
);
const Solution = lazy(() =>
  import("@/components/landing/Solution").then((m) => ({ default: m.Solution }))
);
const FeaturesGrid = lazy(() =>
  import("@/components/landing/FeaturesGrid").then((m) => ({
    default: m.FeaturesGrid,
  }))
);
const ArchitectureSection = lazy(() =>
  import("@/components/landing/ArchitectureSection").then((m) => ({
    default: m.ArchitectureSection,
  }))
);
const BentoDemo = lazy(() =>
  import("@/components/landing/BentoDemo").then((m) => ({
    default: m.BentoDemo,
  }))
);
const UseCases = lazy(() =>
  import("@/components/landing/UseCases").then((m) => ({ default: m.UseCases }))
);
const RoadmapSection = lazy(() =>
  import("@/components/landing/RoadmapSection").then((m) => ({
    default: m.RoadmapSection,
  }))
);
const CTA = lazy(() =>
  import("@/components/landing/CTA").then((m) => ({ default: m.CTA }))
);

function SectionFallback() {
  return <div className="min-h-[40vh]" aria-hidden />;
}

export function LandingPage() {
  return (
    <>
      <FloatingIconsHero
        className="pt-16"
        title="The CNCF Landscape, in your AI assistant."
        subtitle="One server. 2,400+ projects. MCP tools, resources, prompts. STDIO, SSE, or Streamable HTTP."
        ctaText="Get started"
        ctaHref={DOCS_URL}
        secondaryCtaText="GitHub"
        secondaryCtaHref={GITHUB_URL}
        icons={heroFloatingIcons}
      />
      <Suspense fallback={<SectionFallback />}>
        <Problem />
        <Solution />
        <FeaturesGrid />
        <ArchitectureSection />
        <BentoDemo />
        <UseCases />
        <RoadmapSection />
        <CTA />
      </Suspense>
    </>
  );
}
