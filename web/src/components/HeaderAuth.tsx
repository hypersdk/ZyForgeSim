"use client";

import { useRouter } from "next/navigation";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui";

export function HeaderAuth() {
  const router = useRouter();
  const [username, setUsername] = useState<string | null>(null);

  useEffect(() => {
    fetch("/api/auth/me", { credentials: "include" })
      .then((res) => res.json())
      .then((data: { enabled?: boolean; authenticated?: boolean }) => {
        setUsername(data.authenticated ? "Admin" : null);
      })
      .catch(() => setUsername(null));
  }, []);

  async function logout() {
    await fetch("/api/auth/logout", { method: "POST", credentials: "include" });
    router.replace("/login");
    router.refresh();
  }

  if (!username) return null;

  return (
    <div className="header-actions">
      <div className="user-pill">
        <span className="user-avatar">{username.slice(0, 1).toUpperCase()}</span>
        <span>{username}</span>
      </div>
      <Button variant="secondary" onClick={logout}>
        Sign out
      </Button>
    </div>
  );
}
