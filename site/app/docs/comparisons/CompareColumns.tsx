export function CompareSection({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <section className="mt-10">
      <h2 className="mb-4 text-2xl font-semibold tracking-tight">{title}</h2>
      <div className="grid gap-4 md:grid-cols-2">{children}</div>
    </section>
  );
}

export function CompareCard({
  side,
  children,
}: {
  side: "guest" | "tinyspec";
  children: React.ReactNode;
}) {
  const isGuest = side === "guest";
  return (
    <div
      className={`rounded-lg border p-5 ${
        isGuest
          ? "border-border bg-code-bg"
          : "border-accent/30 bg-accent/5"
      }`}
    >
      <div
        className={`mb-3 inline-block rounded-full px-2.5 py-0.5 text-xs font-semibold ${
          isGuest
            ? "bg-muted/20 text-muted"
            : "bg-accent/10 text-accent"
        }`}
      >
        {isGuest ? "Alternative" : "tinyspec"}
      </div>
      <div className="compare-card-content text-sm leading-7 text-foreground/90 [&>p:last-child]:mb-0 [&>p]:mb-3">
        {children}
      </div>
    </div>
  );
}
