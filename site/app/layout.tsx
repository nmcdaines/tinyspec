import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import Script from "next/script";
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
      <head>
        <Script
          src="https://www.googletagmanager.com/gtag/js?id=G-NWTKJHH7X2"
          strategy="afterInteractive"
        />
        <Script id="google-analytics" strategy="afterInteractive">
          {`
            window.dataLayer = window.dataLayer || [];
            function gtag(){dataLayer.push(arguments);}
            gtag('js', new Date());
            gtag('config', 'G-NWTKJHH7X2');
          `}
        </Script>
      </head>
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased`}
      >
        {children}
      </body>
    </html>
  );
}
