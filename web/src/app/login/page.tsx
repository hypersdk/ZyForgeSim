import { Suspense } from "react";
import { LoginForm } from "./LoginForm";

export default function LoginPage() {
  return (
    <Suspense fallback={<div className="text-hs-muted">Loading…</div>}>
      <LoginForm />
    </Suspense>
  );
}
