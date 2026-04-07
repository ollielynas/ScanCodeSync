/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        sans: ["Outfit", "Segoe UI", "sans-serif"],
        mono: ["IBM Plex Mono", "Consolas", "monospace"],
      },
      boxShadow: {
        soft: "0 12px 40px -20px rgba(10, 30, 45, 0.55)",
      },
      colors: {
        brand: {
          50: "#f3f7f5",
          100: "#dbe9e2",
          500: "#2b8c6f",
          700: "#1d5f4b",
          900: "#12372d",
        },
      },
    },
  },
  plugins: [],
};
