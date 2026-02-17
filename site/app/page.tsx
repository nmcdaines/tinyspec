import Hero from "./components/Hero";
import Features from "./components/Features";
import Install from "./components/Install";

export default function Home() {
  return (
    <main className="min-h-screen">
      <Hero />
      <Features />
      <Install />
    </main>
  );
}
