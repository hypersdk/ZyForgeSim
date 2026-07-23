import clsx from "clsx";
import Link from "next/link";
import type { ButtonHTMLAttributes, InputHTMLAttributes, SelectHTMLAttributes } from "react";

export function Card({
  title,
  children,
  className,
}: {
  title?: string;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <section className={clsx("zyvor-card", className)}>
      {title ? (
        <h2 className="mb-3 text-xs font-semibold uppercase tracking-widest text-hs-muted">{title}</h2>
      ) : null}
      {children}
    </section>
  );
}

export function MetricTile({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-hs border border-hs-border border-l-4 border-l-hs-accent bg-hs-bg/50 p-3">
      <div className="text-xs uppercase tracking-wide text-hs-muted">{label}</div>
      <div className="mt-1 font-mono text-xl font-semibold text-hs-heading">{value}</div>
    </div>
  );
}

export function StatusBadge({ status }: { status: string }) {
  const color =
    status === "completed"
      ? "bg-hs-success/20 text-hs-success-light border border-hs-success/30"
      : status === "running"
        ? "bg-hs-indigo/20 text-hs-purple-light border border-hs-indigo/30"
        : status === "failed"
          ? "bg-hs-error/20 text-hs-error-light border border-hs-error/30"
          : "bg-hs-surface text-hs-muted border border-hs-border";
  return (
    <span className={clsx("rounded-hs px-2 py-0.5 text-xs font-medium capitalize", color)}>{status}</span>
  );
}

export function Button({
  variant = "primary",
  className,
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & { variant?: "primary" | "secondary" }) {
  return (
    <button
      className={clsx(variant === "primary" ? "zyvor-btn-primary" : "zyvor-btn-secondary", className)}
      {...props}
    />
  );
}

export function AppLink({
  href,
  className,
  children,
}: {
  href: string;
  className?: string;
  children: React.ReactNode;
}) {
  return (
    <Link href={href} className={clsx("zyvor-link text-sm", className)}>
      {children}
    </Link>
  );
}

export function Input(props: InputHTMLAttributes<HTMLInputElement>) {
  return <input className="zyvor-input" {...props} />;
}

export function Select(props: SelectHTMLAttributes<HTMLSelectElement>) {
  return <select className="zyvor-input" {...props} />;
}
