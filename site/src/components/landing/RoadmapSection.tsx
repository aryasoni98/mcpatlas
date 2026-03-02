import { motion } from "framer-motion";
import { ROADMAP_PHASES, SUCCESS_METRICS } from "@/lib/constants";
import { Section } from "@/components/ui/Section";
import { Heading, Text } from "@/components/ui/Typography";
import {
  staggerContainer,
  staggerItem,
  reducedStaggerContainer,
  reducedStaggerItem,
  viewportOnce,
} from "@/components/motion";
import { useReducedMotion } from "@/components/motion";

const TOTAL_PHASES = ROADMAP_PHASES.length;

export function RoadmapSection() {
  const reduced = useReducedMotion();
  const containerVariants = reduced ? reducedStaggerContainer : staggerContainer;
  const itemVariants = reduced ? reducedStaggerItem : staggerItem;

  return (
    <Section id="roadmap">
      <Heading as="h2">Roadmap</Heading>
      <Text className="mt-4" muted>
        Phases aligned with the project roadmap. See the docs for details.
      </Text>
      <motion.ul
        className="mt-10 space-y-6"
        variants={containerVariants}
        initial="hidden"
        whileInView="visible"
        viewport={viewportOnce}
      >
        {ROADMAP_PHASES.map((phase, i) => (
          <motion.li
            key={phase.name}
            variants={itemVariants}
            className="relative rounded-xl border border-border bg-bg-elevated p-6 shadow-sm"
          >
            <div className="flex flex-wrap items-center justify-between gap-2">
              <h3 className="font-semibold text-foreground">{phase.name}</h3>
              <span className="text-xs font-medium text-muted-foreground">
                Phase {i + 1} of {TOTAL_PHASES}
              </span>
            </div>
            <p className="mt-1 text-sm text-muted-foreground">{phase.goal}</p>
            {phase.done.length > 0 && (
              <ul className="mt-4 space-y-1.5 text-sm" role="list">
                {phase.done.map((item) => (
                  <li key={item} className="flex items-center gap-2">
                    <span className="text-accent-success" aria-hidden="true">
                      ✓
                    </span>
                    <span className="text-foreground/90">{item}</span>
                  </li>
                ))}
              </ul>
            )}
            {phase.upcoming.length > 0 && (
              <ul className="mt-2 space-y-1 text-sm text-muted-foreground" role="list">
                {phase.upcoming.map((item) => (
                  <li key={item}>· {item}</li>
                ))}
              </ul>
            )}
          </motion.li>
        ))}
      </motion.ul>

      <div className="mt-16">
        <h3 className="text-lg font-semibold text-foreground">
          Success metrics (targets)
        </h3>
        <p className="mt-1 text-sm text-muted-foreground">
          Goals aligned with the project roadmap. See ROADMAP.md for details.
        </p>
        <div className="mt-6 overflow-x-auto rounded-xl border border-border bg-bg-elevated">
          <table className="w-full min-w-[420px] text-left text-sm">
            <thead>
              <tr className="border-b border-border">
                <th className="px-4 py-3 font-medium text-foreground">
                  Metric
                </th>
                <th className="px-4 py-3 font-medium text-muted-foreground">
                  3 mo
                </th>
                <th className="px-4 py-3 font-medium text-muted-foreground">
                  6 mo
                </th>
                <th className="px-4 py-3 font-medium text-muted-foreground">
                  12 mo
                </th>
              </tr>
            </thead>
            <tbody>
              {SUCCESS_METRICS.map((row) => (
                <tr
                  key={row.metric}
                  className="border-b border-border last:border-b-0"
                >
                  <td className="px-4 py-3 font-medium text-foreground">
                    {row.metric}
                  </td>
                  <td className="px-4 py-3 text-muted-foreground">
                    {row["3mo"]}
                  </td>
                  <td className="px-4 py-3 text-muted-foreground">
                    {row["6mo"]}
                  </td>
                  <td className="px-4 py-3 text-muted-foreground">
                    {row["12mo"]}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </Section>
  );
}
