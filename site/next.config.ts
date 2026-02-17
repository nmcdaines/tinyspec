import createMDX from "@next/mdx";

const withMDX = createMDX({});

export default withMDX({
  output: "export",
  pageExtensions: ["ts", "tsx", "md", "mdx"],
});
