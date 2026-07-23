import clsx from "clsx";

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
    <section className={clsx("rounded-lg border border-slate-700 bg-slate-900/60 p-4", className)}>
      {title ? <h2 className="mb-3 text-sm font-semibold uppercase tracking-wide text-slate-300">{title}</h2> : null}
      {children}
    </section>
  );
}

export function MetricTile({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-md border border-slate-700 bg-slate-950/50 p-3">
      <div className="text-xs text-slate-400">{label}</div>
      <div className="mt-1 text-xl font-semibold text-white">{value}</div>
    </div>
  );
}

export function StatusBadge({ status }: { status: string }) {
  const color =
    status === "completed"
      ? "bg-green-900 text-green-200"
      : status === "running"
        ? "bg-blue-900 text-blue-200"
        : status === "failed"
          ? "bg-red-900 text-red-200"
          : "bg-slate-800 text-slate-200";
  return <span className={clsx("rounded px-2 py-0.5 text-xs font-medium", color)}>{status}</span>;
}
