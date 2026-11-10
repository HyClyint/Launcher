/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ['./index.html', './src/**/*.{ts,tsx}'],
    theme: {
        extend: {
            colors: {
                ink: {
                    DEFAULT: 'rgb(var(--ink) / <alpha-value>)',
                    soft: 'rgb(var(--ink-soft) / <alpha-value>)',
                    mute: 'rgb(var(--ink-mute) / <alpha-value>)',
                },
                surface: 'rgb(var(--surface) / <alpha-value>)',
                accent: {
                    DEFAULT: 'rgb(var(--accent) / <alpha-value>)',
                    strong: 'rgb(var(--accent-strong) / <alpha-value>)',
                },
                danger: 'rgb(var(--danger) / <alpha-value>)',
            },
            fontFamily: {
                sans: ['Inter', 'Segoe UI', 'system-ui', 'sans-serif'],
            },
            backgroundImage: {
                scene: 'url("./assets/background.png")',
            },
        },
    },
    plugins: [],
};
