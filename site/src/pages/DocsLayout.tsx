import { Outlet, Link, useParams, useLocation } from "react-router-dom";
import { motion } from "framer-motion";
import { DOC_NAV, type DocNavItem } from "@/lib/docs";

function NavLink({
  href,
  isActive,
  label,
}: {
  href: string;
  isActive: boolean;
  label: string;
}) {
  return (
    <li className="relative">
      {isActive && (
        <motion.span
          layoutId="docs-nav-active"
          className="absolute left-0 top-0 bottom-0 w-0.5 rounded-full bg-blue-500"
          transition={{ type: "spring", stiffness: 400, damping: 35 }}
        />
      )}
      <Link
        to={href}
        className={`relative block rounded-r py-1.5 pl-4 pr-2 text-sm transition-colors duration-200 ${
          isActive
            ? "font-medium text-cyan-400"
            : "text-neutral-400 hover:text-neutral-200"
        }`}
      >
        {label}
      </Link>
    </li>
  );
}

export function DocsLayout() {
  const { slug } = useParams<{ slug?: string }>();
  const location = useLocation();
  const hash = location.hash.slice(1).toLowerCase();

  const isItemActive = (item: DocNavItem) => {
    if (slug !== item.slug) return false;
    if (item.anchor) return hash === item.anchor;
    return !hash || hash === "";
  };

  return (
    <div className="flex min-h-[60vh] flex-col md:flex-row">
      <aside className="w-full border-b border-neutral-800 bg-neutral-900 py-6 md:w-64 md:border-b-0 md:border-r md:py-8">
        <nav className="px-4 md:px-5" aria-label="Docs">
          <ul className="space-y-6">
            {DOC_NAV.map((group) => (
              <li key={group.title}>
                <div className="mb-2 pl-2 text-sm font-bold text-neutral-300">
                  {group.title}
                </div>
                <ul className="space-y-0.5 border-l border-neutral-700/80 pl-2">
                  {group.items.map((item) => {
                    const href = item.anchor
                      ? `/docs/${item.slug}#${item.anchor}`
                      : `/docs/${item.slug}`;
                    const active = isItemActive(item);
                    return (
                      <NavLink
                        key={item.anchor ?? item.slug + item.label}
                        href={href}
                        isActive={active}
                        label={item.label}
                      />
                    );
                  })}
                </ul>
              </li>
            ))}
          </ul>
        </nav>
      </aside>
      <div className="flex-1">
        <Outlet />
      </div>
    </div>
  );
}
