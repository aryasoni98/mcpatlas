import { Section } from "@/components/ui/Section";
import { Heading, Text } from "@/components/ui/Typography";
import { SYSTEM_FACT_STRIP } from "@/lib/constants";

export function Problem() {
  return (
    <Section id="problem">
      <div className="max-w-[40rem]">
        <Heading as="h2">Why this matters</Heading>
        <Text className="mt-4 text-base leading-[1.6]" muted>
          The CNCF Landscape has 2,400+ projects. Choosing the right service
          mesh, runtime, or operator is hard. AI assistants lack structured
          access to project data, maturity, and relationships — so
          recommendations stay generic or outdated.
        </Text>
        <p
          className="mt-6 font-mono text-[0.8125rem] leading-[1.5] text-muted-foreground"
          aria-hidden
        >
          {SYSTEM_FACT_STRIP}
        </p>
      </div>
    </Section>
  );
}
