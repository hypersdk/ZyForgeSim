import { requireAuth } from "@/lib/require-auth";
import { DashboardHome } from "./DashboardHome";

export default async function HomePage() {
  await requireAuth("/");
  return <DashboardHome />;
}
