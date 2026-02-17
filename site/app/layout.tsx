import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "tinyspec — Spec-driven development with Claude Code",
  description:
    "A tiny framework for writing structured specifications that turn ideas into tracked implementation plans — powered by AI.",
  openGraph: {
    title: "tinyspec — Spec-driven development with Claude Code",
    description:
      "A tiny framework for writing structured specifications that turn ideas into tracked implementation plans — powered by AI.",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "tinyspec — Spec-driven development with Claude Code",
    description:
      "A tiny framework for writing structured specifications that turn ideas into tracked implementation plans — powered by AI.",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased`}
      >
        {children}
      </body>
    </html>
  );
}
