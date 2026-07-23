import { NextRequest, NextResponse } from "next/server";
import {
  COOKIE_NAME,
  SESSION_MAX_AGE,
  createSessionToken,
  verifyCredentials,
} from "@/lib/auth";

export async function POST(request: NextRequest) {
  let username = "";
  let password = "";
  try {
    const body = (await request.json()) as { username?: string; password?: string };
    username = body.username ?? "";
    password = body.password ?? "";
  } catch {
    return NextResponse.json({ detail: "Invalid request body" }, { status: 400 });
  }

  if (!verifyCredentials(username, password)) {
    return NextResponse.json({ detail: "Invalid username or password" }, { status: 401 });
  }

  const token = await createSessionToken();
  const response = NextResponse.json({ ok: true, username: username.trim() });
  response.cookies.set({
    name: COOKIE_NAME,
    value: token,
    httpOnly: true,
    sameSite: "lax",
    secure: process.env.NODE_ENV === "production",
    path: "/",
    maxAge: SESSION_MAX_AGE,
  });
  return response;
}
