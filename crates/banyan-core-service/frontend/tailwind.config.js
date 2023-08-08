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
		darkTheme: 'dark',
	},
	theme: {
		extend: {
			animation: {
				marquee: 'marquee 20s linear infinite',
			},
			aspectRatio: {
				'22/23': '22 / 23',
			},
			boxShadow: {
				blue: '0px 4px 0px rgba(37, 99, 235, 0.35)',
			},
			height: {
				navbar: '72px',
			},
			colors: {
				blue: {
					primary: '#3E8CDA',
					100: '#DBEAFE',
					600: '#2563EB',
					900: '#1E3A8A',
				},
				navigation: {
					primary: '#FFECC5',
					secondary: '#fff8e7',
					text: '#30374F',
					textSecondary: '#7D89B0',
					border: '#EFC163',
				},
			},
			fontFamily: {
				sans: ['Inter'],
			},
			fontSize: {
				xxs: ['12px', { lineHeight: '18px' }],
				xs: ['14px', { lineHeight: '20px' }],
				sm: ['16px', { lineHeight: '24px' }],
				m: ['18px', { lineHeight: '26px' }],
				deviceCode: ['30px', { lineHeight: '36px' }],
				xxs: ['12px', { lineHeight: '16px' }],
				base: ['20px', { lineHeight: '28px' }],
				lg: ['24px', { lineHeight: '32px' }],
				xl: ['38px', { lineHeight: '48px' }],
				'2xl': ['50px', { lineHeight: '62px' }],
			},
			keyframes: {
				marquee: {
					'0%': { transform: 'translateX(102%)' },
					'100%': { transform: 'translateX(-100vw)' },
				},
			},
			screens: {
				xs: '360px',
			},
			width: {
				navbar: '280px',
				wideModal: '471px',
			},
			zIndex: {
				max: '1000', // High enough to appear above the modal(999)
			},
		},
	},
	purge: {
		options: {
			safelist: ['alert-success', 'alert-error', 'alert-info', 'alert-warning'],
		},
	},
};
