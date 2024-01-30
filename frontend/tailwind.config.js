/** @type {import('tailwindcss').Config} */
export default {
	content: [
	  "./index.html",
	  "./src/**/*.{vue,js,ts,jsx,tsx}",
	],
	theme: {
	  extend: {
		gridTemplateColumns: {
			'19': 'repeat(19, minmax(0, 1fr))',
		}
	  },
	},
	plugins: [],
}