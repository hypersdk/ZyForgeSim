import clsx from "clsx";
import Link from "next/link";
import type { ButtonHTMLAttributes, InputHTMLAttributes, SelectHTMLAttributes } from "react";

export function Card({
  title,
  description,
  children,
  className,
  variant = "default",
}: {
  title?: string;
  description?: string;
  children: React.ReactNode;
  className?: string;
  variant?: "default" | "action";
}) {
  return (
    <section className={clsx("zyvor-card", variant === "action" && "action-card", className)}>
      {title ? (
        <div className="card-header">
          <div className="card-title-wrap">
            <h2 className="card-title">{title}</h2>
            {description ? <p className="card-description">{description}</p> : null}
          </div>
        </div>
      ) : null}
      {children}
    </section>
  );
}

export function PageHero({
  kicker,
  title,
  subtitle,
  actions,
}: {
  kicker?: string;
  title: string;
  subtitle?: string;
  actions?: React.ReactNode;
}) {
  return (
    <div className="page-hero">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          {kicker ? <p className="page-hero-kicker">{kicker}</p> : null}
          <h1 className="page-hero-title">{title}</h1>
          {subtitle ? <p className="page-hero-subtitle">{subtitle}</p> : null}
        </div>
        {actions ? <div className="flex gap-2">{actions}</div> : null}
      </div>
    </div>
  );
}

export function FormField({
  label,
  children,
  className,
}: {
  label: string;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <label className={clsx("form-field", className)}>
      <span className="form-label">{label}</span>
      {children}
    </label>
  );
}

export function EmptyState({
  title,
  text,
  children,
}: {
  title: string;
  text: string;
  children?: React.ReactNode;
}) {
  return (
    <div className="empty-state">
      <p className="empty-state-title">{title}</p>
      <p className="empty-state-text">{text}</p>
      {children}
    </div>
  );
}

export function MetricTile({ label, value }: { label: string; value: string }) {
  return (
    <div className="metric-tile">
      <div className="metric-tile-label">{label}</div>
      <div className="metric-tile-value">{value}</div>
    </div>
  );
}

const statusStyles: Record<string, string> = {
  completed: "bg-hs-success/15 text-hs-success-light border border-hs-success/25",
  running: "bg-hs-indigo/15 text-hs-purple-light border border-hs-indigo/25",
  failed: "bg-hs-error/15 text-hs-error-light border border-hs-error/25",
  pending: "bg-hs-surface text-hs-muted border border-hs-border",
};

export function StatusBadge({ status }: { status: string }) {
  return (
    <span className={clsx("status-badge", statusStyles[status] ?? statusStyles.pending)}>{status}</span>
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
  showArrow = true,
}: {
  href: string;
  className?: string;
  children: React.ReactNode;
  showArrow?: boolean;
}) {
  return (
    <Link href={href} className={clsx("zyvor-link", className)}>
      {children}
      {showArrow ? <span aria-hidden="true">→</span> : null}
    </Link>
  );
}

export function Input(props: InputHTMLAttributes<HTMLInputElement>) {
  return <input className="zyvor-input w-full" {...props} />;
}

export function Select(props: SelectHTMLAttributes<HTMLSelectElement>) {
  return <select className="zyvor-input w-full min-w-[12rem]" {...props} />;
}
