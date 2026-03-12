export interface NavItem {
  title: string;
  slug: string;
  children?: NavItem[];
}

export const docsNav: NavItem[] = [
  { title: "Getting Started", slug: "getting-started" },
  { title: "Concepts", slug: "concepts" },
  { title: "CLI Reference", slug: "cli-reference" },
  { title: "Claude Code Integration", slug: "claude-code" },
  { title: "Configuration", slug: "configuration" },
  { title: "Dashboard", slug: "dashboard" },
  {
    title: "Comparisons",
    slug: "comparisons",
    children: [
      { title: "vs OpenSpec", slug: "comparisons/openspec" },
      { title: "vs SpecKit", slug: "comparisons/speckit" },
      { title: "vs BMad", slug: "comparisons/bmad" },
      { title: "vs GSD", slug: "comparisons/gsd" },
    ],
  },
];

export function flatNav(): NavItem[] {
  const items: NavItem[] = [];
  for (const item of docsNav) {
    items.push(item);
    if (item.children) {
      items.push(...item.children);
    }
  }
  return items;
}
