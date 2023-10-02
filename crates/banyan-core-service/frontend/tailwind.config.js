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
				filePreview: '70%',
			},
			padding: {
				keyManagement: '72px',
				2.5: '10px',
				1.5: '6px',
			},
			minWidth: {
				navbar: '280px',
				login: '590px',
			},
			maxWidth: {
				filePreview: '952px',
			},
			colors: {
				blue: {
					primary: 'var(--blue-primary)',
					hover: 'var(--blue-hover)',
				},
				error: 'var(--error)',
				errorBanner: 'var(--errorBanner)',
				login: 'var(--login)',
				navigation: {
					primary: 'var(--navigation-primary)',
					secondary: 'var(--navigation-secondary)',
					text: 'var(--navigation-text)',
					textSecondary: 'var(--navigation-textSecondary)',
					border: 'var(--navigation-border)',
				},
				table: {
					cellBackground: 'var(--table-cellBackground)',
					headBackground: 'var(--table-headBackground)',
				},
				'gray-200': 'var(--gray-200)',
				'gray-600': 'var(--gray-600)',
				'gray-800': 'var(--gray-800)',
				'gray-900': 'var(--gray-900)',
				border: 'var(--border)',
				disabled: 'var(--disabled)',
				hover: 'var(--hover-background)',
				mainBackground: 'var(--background)'
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
