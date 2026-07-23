"use client";

import { useSearchParams } from "next/navigation";
import { FormEvent, useState } from "react";
import { Button, Input } from "@/components/ui";
import { DEFAULT_PASSWORD, DEFAULT_USERNAME } from "@/lib/auth";

function EyeIcon({ open }: { open: boolean }) {
  if (open) {
    return (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75" className="h-5 w-5">
        <path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7-10-7-10-7Z" />
        <circle cx="12" cy="12" r="3" />
      </svg>
    );
  }
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75" className="h-5 w-5">
      <path d="M3 3l18 18" />
      <path d="M10.58 10.58A3 3 0 0 0 12 15a3 3 0 0 0 2.42-4.42" />
      <path d="M9.88 5.1A10.8 10.8 0 0 1 12 5c6.5 0 10 7 10 7a17.4 17.4 0 0 1-2.12 3.17" />
      <path d="M6.12 6.12A17.4 17.4 0 0 0 4 12s3.5 7 10 7a10.8 10.8 0 0 0 3.9-.72" />
    </svg>
  );
}

export function LoginForm() {
  const searchParams = useSearchParams();
  const nextPath = searchParams.get("next") || "/";
  const [username, setUsername] = useState(DEFAULT_USERNAME);
  const [password, setPassword] = useState(DEFAULT_PASSWORD);
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  async function onSubmit(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError("");
    try {
      const res = await fetch("/api/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username, password }),
        credentials: "include",
      });
      if (!res.ok) {
        const data = (await res.json().catch(() => null)) as { detail?: string } | null;
        setError(data?.detail ?? `Login failed (${res.status})`);
        return;
      }
      window.location.assign(nextPath.startsWith("/") ? nextPath : "/");
    } catch {
      setError("Unable to reach the server");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="mx-auto flex w-full max-w-[420px] flex-col justify-center py-10">
      <div className="login-panel">
        <div className="login-panel-header">
          <p className="login-panel-kicker">ForgeSim Dashboard</p>
          <h1 className="login-panel-title">Welcome back</h1>
          <p className="login-panel-subtitle">Sign in to monitor simulations, replay runs, and compare schedulers.</p>
        </div>

        <p className="login-hint">
          Default: <span className="font-mono">{DEFAULT_USERNAME}</span> /{" "}
          <span className="font-mono">{DEFAULT_PASSWORD}</span>
        </p>

        <form className="login-form" onSubmit={onSubmit}>
          <div className="login-field">
            <label className="login-label" htmlFor="username">
              Username
            </label>
            <Input
              id="username"
              className="login-input w-full"
              type="text"
              autoComplete="username"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              required
            />
          </div>

          <div className="login-field">
            <label className="login-label" htmlFor="password">
              Password
            </label>
            <div className="login-password-wrap">
              <Input
                id="password"
                className="login-input login-input-password w-full"
                type={showPassword ? "text" : "password"}
                autoComplete="current-password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
              />
              <button
                type="button"
                className="login-password-toggle"
                onClick={() => setShowPassword((v) => !v)}
                aria-label={showPassword ? "Hide password" : "Show password"}
              >
                <EyeIcon open={showPassword} />
              </button>
            </div>
          </div>

          {error ? <p className="login-error">{error}</p> : null}

          <Button type="submit" className="login-submit w-full" disabled={busy || !username || !password}>
            {busy ? "Signing in…" : "Sign in"}
          </Button>
        </form>
      </div>
    </div>
  );
}
