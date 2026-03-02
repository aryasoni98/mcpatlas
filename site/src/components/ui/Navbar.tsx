import { useState, useEffect } from "react";
import { Link } from "react-router-dom";
import { useScroll } from "framer-motion";
import { NAV_LINKS } from "@/lib/constants";
import { Container } from "./Container";

export function Navbar() {
  const [mobileOpen, setMobileOpen] = useState(false);
  const [scrolled, setScrolled] = useState(false);
  const { scrollY } = useScroll();

  useEffect(() => {
    const unsub = scrollY.on("change", (v) => setScrolled(v > 80));
    return () => unsub();
  }, [scrollY]);

  return (
    <nav
      className={`sticky top-0 z-50 border-b border-border backdrop-blur-xl transition-[background-color] duration-200 ${scrolled ? "bg-white/95 dark:bg-neutral-950/95" : "bg-white/70 dark:bg-neutral-950/70"}`}
      aria-label="Main"
    >
      <Container>
        <div className="flex h-14 items-center justify-between md:h-16">
          <Link
            to={NAV_LINKS[0].href === "/" ? "/" : NAV_LINKS[0].href}
            className="text-lg font-semibold text-foreground"
          >
            CNCF MCP
          </Link>
          <ul className="hidden gap-8 md:flex">
            {NAV_LINKS.map((link) =>
              link.external ? (
                <li key={link.label}>
                  <a
                    href={link.href}
                    className="text-muted-foreground hover:text-foreground transition-colors"
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    {link.label}
                  </a>
                </li>
              ) : (
                <li key={link.label}>
                  <Link
                    to={link.href}
                    className="text-muted-foreground hover:text-foreground transition-colors"
                    onClick={() => setMobileOpen(false)}
                  >
                    {link.label}
                  </Link>
                </li>
              )
            )}
          </ul>
          <button
            type="button"
            className="rounded p-2 md:hidden"
            onClick={() => setMobileOpen(!mobileOpen)}
            aria-expanded={mobileOpen}
            aria-controls="mobile-menu"
            aria-label="Toggle menu"
          >
            <span className="sr-only">Toggle menu</span>
            <svg
              className="h-6 w-6"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              aria-hidden
            >
              {mobileOpen ? (
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M6 18L18 6M6 6l12 12"
                />
              ) : (
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M4 6h16M4 12h16M4 18h16"
                />
              )}
            </svg>
          </button>
        </div>
        <div
          id="mobile-menu"
          className={`md:hidden ${mobileOpen ? "block border-t border-border py-4" : "hidden"}`}
        >
          <ul className="flex flex-col gap-4">
            {NAV_LINKS.map((link) =>
              link.external ? (
                <li key={link.label}>
                  <a
                    href={link.href}
                    className="block text-muted-foreground hover:text-foreground transition-colors"
                    target="_blank"
                    rel="noopener noreferrer"
                    onClick={() => setMobileOpen(false)}
                  >
                    {link.label}
                  </a>
                </li>
              ) : (
                <li key={link.label}>
                  <Link
                    to={link.href}
                    className="block text-muted-foreground hover:text-foreground transition-colors"
                    onClick={() => setMobileOpen(false)}
                  >
                    {link.label}
                  </Link>
                </li>
              )
            )}
          </ul>
        </div>
      </Container>
    </nav>
  );
}
