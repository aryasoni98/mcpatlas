import { Section } from "@/components/ui/Section";
import { Heading, Text } from "@/components/ui/Typography";
import { SYSTEM_FACT_STRIP } from "@/lib/constants";

export function Problem() {
  return (
    <Section id="problem">
      <div className="max-w-[40rem]">
        <Heading as="h2">Why this matters</Heading>
        <Text className="mt-4 text-base leading-[1.6]" muted>
          The CNCF Landscape has 2,400+ projects across thousands of repos.
          AI assistants lack structured access to project data, relationships,
          and issue context — so contributions stay manual and tool selection
          stays guesswork. Paste a GitHub issue URL and get a resolution brief,
          or ask which graduated service mesh supports mTLS and get a
          data-backed answer.
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
