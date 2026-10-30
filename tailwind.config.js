/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        blueprint: {
          dark: '#030d1a',
          grid: '#0a223e',
          cyan: '#00e5ff',
          blue: '#0284c7',
          neon: '#38bdf8',
          accent: '#3b82f6',
          alert: '#f43f5e',
          energy: '#a855f7',
        }
      },
      fontFamily: {
        mono: ['Courier New', 'monospace', 'ui-monospace'],
      }
    },
  },
  plugins: [],
}

