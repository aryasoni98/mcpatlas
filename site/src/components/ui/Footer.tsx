import { NAV_LINKS } from "@/lib/constants";
import { Container } from "./Container";

export function Footer() {
  const year = new Date().getFullYear();

  return (
    <footer className="border-t border-neutral-200 bg-neutral-50 dark:border-neutral-800 dark:bg-neutral-900/50">
      <Container>
        <div className="flex flex-col gap-6 py-12 md:flex-row md:items-center md:justify-between">
          <div className="flex flex-wrap gap-6">
            {NAV_LINKS.slice(1).map((link) => (
              <a
                key={link.label}
                href={link.href}
                className="text-neutral-600 hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-white"
                {...(link.external
                  ? { target: "_blank", rel: "noopener noreferrer" }
                  : {})}
              >
                {link.label}
              </a>
            ))}
          </div>
          <p className="text-sm text-neutral-500 dark:text-neutral-400">
            © {year} CNCF MCP. Open source.
          </p>
        </div>
      </Container>
    </footer>
  );
}
