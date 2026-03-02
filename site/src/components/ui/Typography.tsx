interface HeadingProps {
  as?: "h1" | "h2" | "h3";
  children: React.ReactNode;
  className?: string;
}

export function Heading({
  as: Comp = "h2",
  children,
  className = "",
}: HeadingProps) {
  const size =
    Comp === "h1"
      ? "text-4xl md:text-6xl font-bold tracking-tight"
      : Comp === "h2"
        ? "text-3xl md:text-4xl font-bold tracking-tight"
        : "text-xl md:text-2xl font-semibold";
  return (
    <Comp
      className={`text-neutral-900 dark:text-white ${size} ${className}`.trim()}
    >
      {children}
    </Comp>
  );
}

interface TextProps {
  children: React.ReactNode;
  className?: string;
  muted?: boolean;
}

export function Text({
  children,
  className = "",
  muted = false,
}: TextProps) {
  return (
    <p
      className={`text-base md:text-lg ${
        muted
          ? "text-neutral-600 dark:text-neutral-400"
          : "text-neutral-700 dark:text-neutral-300"
      } ${className}`.trim()}
    >
      {children}
    </p>
  );
}

interface CodeProps {
  children: React.ReactNode;
  className?: string;
}

export function Code({ children, className = "" }: CodeProps) {
  return (
    <code
      className={`rounded bg-neutral-100 dark:bg-neutral-800 px-1.5 py-0.5 font-mono text-sm ${className}`.trim()}
    >
      {children}
    </code>
  );
}
