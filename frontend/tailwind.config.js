module.exports = {
  content: [
    "./src/**/*.{js,jsx,ts,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'light-blue': {
          500: '#3B82F6',
        },
      },
    },
  },
  plugins: [
    require('@tailwindcss/forms'),
  ],
} 