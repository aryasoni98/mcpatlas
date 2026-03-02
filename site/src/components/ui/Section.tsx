import { motion, type Variants } from "framer-motion";
import { sectionReveal, viewportOnce } from "@/components/motion/sectionVariants";
import { useReducedMotion } from "@/components/motion";

interface SectionProps {
  id?: string;
  children: React.ReactNode;
  className?: string;
  as?: "section" | "div";
  animate?: boolean;
}

const reducedReveal: Variants = {
  hidden: { opacity: 0 },
  visible: { opacity: 1, transition: { duration: 0.3 } },
};

export function Section({
  id,
  children,
  className = "",
  as: Comp = "section",
  animate = true,
}: SectionProps) {
  const reduced = useReducedMotion();
  const variants = reduced ? reducedReveal : sectionReveal;

  const content = animate ? (
    <motion.div
      variants={variants}
      initial="hidden"
      whileInView="visible"
      viewport={viewportOnce}
    >
      {children}
    </motion.div>
  ) : (
    children
  );

  return (
    <Comp id={id} className={`py-16 md:py-20 ${className}`.trim()}>
      <div className="mx-auto max-w-[72rem] px-6 md:px-8">{content}</div>
    </Comp>
  );
}
