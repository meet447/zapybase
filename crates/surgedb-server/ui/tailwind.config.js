/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        surge: {
          orange: "#E57E51",
          black: "#000000",
          white: "#FFFFFF",
          gray: "#F5F5F5",
        }
      },
      fontFamily: {
        mono: ["'JetBrains Mono'", "monospace"],
      }
    },
  },
  plugins: [],
}
