import createMDX from "@next/mdx";

const withMDX = createMDX({
  options: {
    remarkPlugins: ["remark-gfm"],
  },
});

export default withMDX({
  output: "export",
  pageExtensions: ["ts", "tsx", "md", "mdx"],
});
