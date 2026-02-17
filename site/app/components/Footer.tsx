export default function Footer() {
  return (
    <footer className="border-t border-border px-6 py-12">
      <div className="mx-auto flex max-w-5xl flex-col items-center justify-between gap-4 text-sm text-muted sm:flex-row">
        <p>&copy; {new Date().getFullYear()} tinyspec</p>
        <div className="flex gap-6">
          <a
            href="https://github.com/nmcdaines/tinyspec"
            target="_blank"
            rel="noopener noreferrer"
            className="transition-colors hover:text-foreground"
          >
            GitHub
          </a>
          <a
            href="https://crates.io/crates/tinyspec"
            target="_blank"
            rel="noopener noreferrer"
            className="transition-colors hover:text-foreground"
          >
            Crates.io
          </a>
        </div>
      </div>
    </footer>
  );
}
