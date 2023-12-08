// TODO: This needs to be simplified and cleaned up
/** @type {import('tailwindcss').Config} */
module.exports = {
	mode: 'jit',
	content: ['./src/**/*.{js,jsx,ts,tsx}'],
	plugins: [require('daisyui')],
	theme: {
		extend: {
			boxShadow: {
				common: 'var(--shadow)',
			},
			height: {
				navbar: '72px',
				termsAndConditions: '756px'
			},
			width: {
				navbar: '280px',
				modal: '400px',
				takeSnapshotModal: '420px',
				toast: '480px',
				snapshotsModal: '530px',
				keyManagement: '1128px',
				filePreview: '80vw',
				termsAndConditions: '1283px',
				termsAndConditionsText: '1184px'
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
				filePreview: '70vw',
			},
			colors: {
				button: {
					primary: 'var(--highlight-color)',
					highLight: 'var(--button-secondary)',
					disabled: 'var(--disabled)',
				},
				error: 'var(--error)',
				errorBanner: 'var(--error-banner)',
				errorBannerBorder: 'var(--error-banner-border)',
				login: 'var(--login)',
				dragging: 'var(--dragging)',
				draggingBorder: 'var(--draggingBorder)',
				navigation: {
					primary: 'var(--navigation-primary)',
					secondary: 'var(--navigation-secondary)',
					text: 'var(--navigation-text)',
					textSecondary: 'var(--navigation-textSecondary)',
					border: 'var(--navigation-border)',
					separator: 'var(--navigation-separator)',
				},
				table: {
					cellBackground: 'var(--table-cellBackground)',
					headBackground: 'var(--table-headBackground)',
				},
				bucket: {
					bucketIconBackground: 'var(--bucket-icon-background)',
					bucketHoverBackground: 'var(--bucket-hover-background)',
					bucketBorder: 'var(--bucket-border)',
					bucketClasshot: 'var(--bucket-class-hot)',
					bucketClasswarm: 'var(--bucket-class-hot)',
					bucketClasscold: 'var(--bucket-class-cold)',
					actionsBackground: 'var(--bucket-actions-background)',
					actionsText: 'var(--bucket-actions-text)',
				},
				modalBackground: 'var(--modal-background)',
				border: {
					darken: 'var(--darken-border)',
					regular: 'var(--regular-border)',
				},
				text: {
					200: 'var(--text-200)',
					400: 'var(--text-400)',
					600: 'var(--text-600)',
					800: 'var(--text-800)',
					900: 'var(--text-900)',
				},
				'gray-200': 'var(--text-200)',
				hover: 'var(--hover-background)',
				mainBackground: 'var(--main-background)',
				secondaryBackground: 'var(--secondary-background)',
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
