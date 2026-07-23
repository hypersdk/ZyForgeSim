import type { Metadata } from "next";
import { Inter, JetBrains_Mono } from "next/font/google";
import Image from "next/image";
import { HeaderAuth } from "@/components/HeaderAuth";
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
      <body className="app-shell font-sans antialiased">
        <header className="app-header">
          <div className="app-header-inner">
            <a href="/" className="brand-mark">
              <div className="brand-logo-wrap">
                <Image
                  src="/zyvor-logo.png"
                  alt="Zyvor AI Labs"
                  width={120}
                  height={40}
                  className="h-7 w-auto"
                  priority
                />
              </div>
              <div className="hidden sm:block">
                <div className="brand-copy-title">ForgeSim</div>
                <div className="brand-copy-sub">Simulation · Replay · Compare</div>
              </div>
            </a>
            <HeaderAuth />
          </div>
        </header>
        <main className="app-main mx-auto max-w-7xl px-4 sm:px-6">{children}</main>
      </body>
    </html>
  );
}
