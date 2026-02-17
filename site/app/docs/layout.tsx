import Header from "../components/Header";
import Footer from "../components/Footer";

export const metadata = {
  title: "Docs — tinyspec",
};

export default function DocsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <>
      <Header />
      <div className="flex min-h-screen pt-16">
        {children}
      </div>
      <Footer />
    </>
  );
}
