export interface NavItem {
  title: string;
  slug: string;
}

export const docsNav: NavItem[] = [
  { title: "Getting Started", slug: "getting-started" },
  { title: "Concepts", slug: "concepts" },
  { title: "CLI Reference", slug: "cli-reference" },
  { title: "Claude Code Integration", slug: "claude-code" },
  { title: "Configuration", slug: "configuration" },
  { title: "Dashboard", slug: "dashboard" },
];
