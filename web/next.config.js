/** @type {import('next').NextConfig} */
const apiBase = process.env.FORGESIM_API_URL || "http://127.0.0.1:8080";

const nextConfig = {
  async rewrites() {
    // Proxy only ForgeSim backend routes — keep /api/auth/* for Next.js login handlers.
    return [
      { source: "/api/health", destination: `${apiBase}/api/health` },
      { source: "/api/configs", destination: `${apiBase}/api/configs` },
      { source: "/api/runs", destination: `${apiBase}/api/runs` },
      { source: "/api/runs/:path*", destination: `${apiBase}/api/runs/:path*` },
      { source: "/api/compare", destination: `${apiBase}/api/compare` },
      { source: "/ws/:path*", destination: `${apiBase}/ws/:path*` },
    ];
  },
};

module.exports = nextConfig;
