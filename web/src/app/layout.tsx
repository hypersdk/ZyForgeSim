import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "ForgeSim Dashboard",
  description: "Monitor, replay, and compare GPU scheduler simulations",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <header className="border-b border-slate-800 bg-slate-900/80">
          <div className="mx-auto flex max-w-7xl items-center justify-between px-4 py-4">
            <a href="/" className="text-lg font-bold tracking-tight text-white">
              ForgeSim
            </a>
            <span className="text-xs text-slate-400">Simulation · Replay · Compare</span>
          </div>
        </header>
        <main className="mx-auto max-w-7xl px-4 py-6">{children}</main>
      </body>
    </html>
  );
}
