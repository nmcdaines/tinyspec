const comparisons = [
  {
    name: "OpenSpec",
    slug: "openspec",
    description:
      "Multi-artifact workflow orchestration with proposals, specs, designs, and tasks as separate versioned files.",
  },
  {
    name: "SpecKit",
    slug: "speckit",
    description:
      "GitHub's full lifecycle framework with formal requirement IDs, constitutional governance, and cross-artifact analysis.",
  },
  {
    name: "BMad METHOD",
    slug: "bmad",
    description:
      "AI persona-driven process framework with specialized agents (PM, Architect, Developer, QA) guiding each phase.",
  },
  {
    name: "GSD",
    slug: "gsd",
    description:
      "Multi-agent orchestration with parallel execution, context engineering, and wave-based dependency resolution.",
  },
];

export default function Comparisons() {
  return (
    <section className="px-6 py-24">
      <div className="mx-auto max-w-5xl">
        <h2 className="text-center text-3xl font-bold tracking-tight sm:text-4xl">
          How tinyspec compares
        </h2>
        <p className="mx-auto mt-4 max-w-2xl text-center text-muted">
          See how tinyspec stacks up against popular alternatives.{" "}
          <a
            href="/docs/comparisons"
            className="text-accent underline underline-offset-4 hover:text-accent-hover"
          >
            View the full comparison table
          </a>
        </p>
        <div className="mt-12 grid gap-4 sm:grid-cols-2">
          {comparisons.map((item) => (
            <a
              key={item.slug}
              href={`/docs/comparisons/${item.slug}`}
              className="group rounded-xl border border-border p-6 transition-colors hover:border-accent/40 hover:bg-accent/5"
            >
              <div className="flex items-center justify-between">
                <h3 className="text-lg font-semibold">
                  vs {item.name}
                </h3>
                <svg
                  className="h-4 w-4 text-muted transition-transform group-hover:translate-x-1 group-hover:text-accent"
                  fill="none"
                  viewBox="0 0 24 24"
                  strokeWidth={2}
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    d="M13.5 4.5 21 12m0 0-7.5 7.5M21 12H3"
                  />
                </svg>
              </div>
              <p className="mt-2 text-sm leading-relaxed text-muted">
                {item.description}
              </p>
            </a>
          ))}
        </div>
      </div>
    </section>
  );
}
