// TODO: This needs to be simplified and cleaned up
/** @type {import('tailwindcss').Config} */
module.exports = {
	mode: 'jit',
	content: ['./src/**/*.{js,jsx,ts,tsx}'],
	plugins: [require('daisyui')],
	darkMode: ['class', '[data-theme="dark"]'],
	daisyui: {
		styled: true,
		base: true,
		utils: true,
		logs: true,
		rtl: false,
		darkTheme: false,
	},
	purge: {
		options: {
			safelist: ['alert-success', 'alert-error', 'alert-info', 'alert-warning'],
		},
	},
};
