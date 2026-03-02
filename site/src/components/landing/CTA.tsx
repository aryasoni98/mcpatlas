import { Button } from "@/components/ui/button";
import { Section } from "@/components/ui/Section";
import { Heading, Text } from "@/components/ui/Typography";
import { DOCS_URL, GITHUB_URL } from "@/lib/constants";

export function CTA() {
  return (
    <Section id="cta">
      <div className="mx-auto max-w-[36rem] rounded-2xl border border-border bg-muted/50 px-8 py-12 text-center">
        <Heading as="h2">Get started</Heading>
        <Text className="mt-4 text-base" muted>
          Add CNCF MCP to Claude, Cursor, or any MCP client. See the docs for
          install and config.
        </Text>
        <p
          className="mt-4 font-mono text-[0.8125rem] text-muted-foreground"
          aria-hidden
        >
          cncf-mcp --transport stdio
        </p>
        <div className="mt-8 flex flex-wrap justify-center gap-4">
          <Button
            asChild
            size="lg"
            className="bg-accent-brand text-white hover:opacity-90 focus-visible:outline-ring"
          >
            <a href={DOCS_URL}>Documentation</a>
          </Button>
          <Button asChild variant="outline" size="lg">
            <a href={GITHUB_URL}>GitHub</a>
          </Button>
        </div>
      </div>
    </Section>
  );
}
