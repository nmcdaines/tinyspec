import Header from "./components/Header";
import Hero from "./components/Hero";
import Features from "./components/Features";
import Install from "./components/Install";
import Footer from "./components/Footer";

export default function Home() {
  return (
    <>
      <Header />
      <main className="min-h-screen pt-16">
        <Hero />
        <Features />
        <Install />
      </main>
      <Footer />
    </>
  );
}
