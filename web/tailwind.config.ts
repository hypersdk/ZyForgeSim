import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./src/**/*.{js,ts,jsx,tsx,mdx}"],
  theme: {
    extend: {
      colors: {
        idle: "#22c55e",
        training: "#3b82f6",
        inference: "#f97316",
        overloaded: "#ef4444",
      },
    },
  },
  plugins: [],
};
export default config;
