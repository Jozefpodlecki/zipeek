/** @type {import('tailwindcss').Config} */
module.exports = {
	content: ["./src/**/*.{html,rs}"],
	mode: "jit",
	purge: false,
	theme: {
		extend: {
			fontFamily: {
				'roboto': ['Roboto', 'sans-serif'],
				'oswald': ['Oswald', 'sans-serif'],
			}
		}
	},
	plugins: [],
}

