import { useEffect, useState, useRef } from "react";
import { useParams, useLocation, Navigate } from "react-router-dom";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeSlug from "rehype-slug";
import { getDocContent, docSlugs, DOC_TITLES } from "@/lib/docs";
import { Container } from "@/components/ui/Container";

export function DocPage() {
  const { slug } = useParams<{ slug: string }>();
  const location = useLocation();
  const [content, setContent] = useState<string | null | undefined>(undefined);
  const articleRef = useRef<HTMLElement>(null);

  useEffect(() => {
    const s = slug;
    if (s) getDocContent(s).then(setContent);
  }, [slug]);

  useEffect(() => {
    const hash = location.hash.slice(1);
    if (!hash || !articleRef.current) return;
    const el = document.getElementById(hash);
    if (el) el.scrollIntoView({ behavior: "smooth", block: "start" });
  }, [location.hash, content]);

  if (!slug) return <Navigate to="/docs/introduction" replace />;
  if (!docSlugs.includes(slug)) return <Navigate to="/docs/introduction" replace />;
  if (content === undefined) {
    return (
      <div className="py-12">
        <Container>
          <div className="animate-pulse text-neutral-500">Loading…</div>
        </Container>
      </div>
    );
  }
  if (content === null) return <Navigate to="/docs/introduction" replace />;

  const title = DOC_TITLES[slug] ?? slug;

  return (
    <article ref={articleRef} className="py-8">
      <Container>
        <h1 className="mb-8 text-3xl font-bold text-neutral-900 dark:text-white">
          {title}
        </h1>
        <div className="prose prose-neutral dark:prose-invert max-w-none">
          <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeSlug]}>
            {content}
          </ReactMarkdown>
        </div>
      </Container>
    </article>
  );
}
