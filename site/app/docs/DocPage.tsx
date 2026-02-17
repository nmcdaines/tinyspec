import Sidebar from "./Sidebar";
import { docsNav } from "./nav";

export default function DocPage({
  slug,
  children,
}: {
  slug: string;
  children: React.ReactNode;
}) {
  const currentIndex = docsNav.findIndex((item) => item.slug === slug);
  const prev = currentIndex > 0 ? docsNav[currentIndex - 1] : null;
  const next =
    currentIndex < docsNav.length - 1 ? docsNav[currentIndex + 1] : null;

  return (
    <>
      <Sidebar currentSlug={slug} />
      <main className="min-w-0 flex-1 px-6 py-10 md:px-12 lg:px-16">
        <article className="docs-content mx-auto max-w-3xl">{children}</article>
        <nav className="mx-auto mt-12 flex max-w-3xl justify-between border-t border-border pt-6">
          {prev ? (
            <a
              href={`/docs/${prev.slug}`}
              className="text-sm text-muted transition-colors hover:text-accent"
            >
              &larr; {prev.title}
            </a>
          ) : (
            <span />
          )}
          {next ? (
            <a
              href={`/docs/${next.slug}`}
              className="text-sm text-muted transition-colors hover:text-accent"
            >
              {next.title} &rarr;
            </a>
          ) : (
            <span />
          )}
        </nav>
      </main>
    </>
  );
}
