import type { Metadata } from "next";
import { Inter, JetBrains_Mono } from "next/font/google";
import Image from "next/image";
import "./globals.css";

const inter = Inter({
  subsets: ["latin"],
  variable: "--font-inter",
});

const jetbrains = JetBrains_Mono({
  subsets: ["latin"],
  variable: "--font-jetbrains",
});

export const metadata: Metadata = {
  title: "ForgeSim · Zyvor AI Labs",
  description: "Monitor, replay, and compare GPU scheduler simulations",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className={`${inter.variable} ${jetbrains.variable}`}>
      <body className="font-sans antialiased">
        <header className="sticky top-0 z-50 border-b border-hs-border bg-hs-bg/80 shadow-hs-elev-2 backdrop-blur-md">
          <div className="mx-auto flex max-w-7xl items-center justify-between px-4 py-3">
            <a href="/" className="flex items-center gap-3">
              <Image
                src="/zyvor-logo.png"
                alt="Zyvor AI Labs"
                width={140}
                height={47}
                className="h-9 w-auto"
                priority
              />
              <div className="hidden sm:block">
                <div className="text-sm font-semibold text-hs-heading">ForgeSim</div>
                <div className="text-xs text-hs-muted">Simulation · Replay · Compare</div>
              </div>
            </a>
          </div>
        </header>
        <main className="mx-auto max-w-7xl px-4 py-6">{children}</main>
      </body>
    </html>
  );
}
