interface BadgeProps {
  children: React.ReactNode;
  className?: string;
}

export function Badge({ children, className = "" }: BadgeProps) {
  return (
    <span
      className={`inline-flex items-center rounded-full bg-neutral-200 px-2.5 py-0.5 text-xs font-medium text-neutral-800 dark:bg-neutral-700 dark:text-neutral-200 ${className}`.trim()}
    >
      {children}
    </span>
  );
}
