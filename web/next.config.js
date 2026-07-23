/** @type {import('next').NextConfig} */
const nextConfig = {
  async rewrites() {
    return [
      { source: "/api/:path*", destination: "http://localhost:8080/api/:path*" },
      { source: "/ws/:path*", destination: "http://localhost:8080/ws/:path*" },
    ];
  },
};

module.exports = nextConfig;
