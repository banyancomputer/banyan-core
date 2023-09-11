// TODO: This needs to be simplified and cleaned up
/** @type {import('tailwindcss').Config} */
module.exports = {
	mode: 'jit',
	content: ['./src/**/*.{js,jsx,ts,tsx}'],
	plugins: [require('daisyui')],
	theme: {
		extend: {
			boxShadow: {
				blue: '0px 4px 0px rgba(37, 99, 235, 0.35)',
			},
			height: {
				navbar: '72px',
			},
			width: {
				navbar: '280px',
				modal: '400px',
				toast: '480px',
				snapshotsModal: '530px',
				keyManagement: '1128px',
			},
			padding: {
				keyManagement: '72px',
				select: '10px'
			},
			minWidth: {
				navbar: '280px',
			},
			minWidth: {
				navbar: '280px',
				login: '590px',
			},
			colors: {
				blue: {
					primary: '#3E8CDA',
					hover: '#287dd2',
					100: '#DBEAFE',
					600: '#2563EB',
					900: '#1E3A8A',
				},
				login: '#F7F7F7',
				navigation: {
					primary: '#FFECC5',
					secondary: '#fff8e7',
					text: '#30374F',
					textSecondary: '#7D89B0',
					border: '#EFC163',
				},
				table: {
					cellBackground: '#EFF1F5',
					headBackground: '#fcfcfd',
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
				'3xl': ['64px', { lineHeight: '64px' }],
			},
			screens: {
				xs: '360px',
			},
			zIndex: {
				max: '1000',
			},
			borderWidth: {
				1: '1px',
			},
		},
	},
	purge: {
		options: {
			safelist: ['alert-success', 'alert-error', 'alert-info', 'alert-warning'],
		},
	},
};
