/** @type {import('next').NextConfig} */
const nextConfig = {
	images: {
		domains: ['lh3.googleusercontent.com'],
	},
	i18n: {
		locales: ['en', 'fr'],
		localeDetection: false,
	},
	webpack(config) {
		// Since Webpack 5 doesn't enable WebAssembly by default, we should do it manually
		config.experiments = { ...config.experiments, asyncWebAssembly: true };

		return config;
	},
	eslint: {
		ignoreDuringBuilds: true,
	},
};

module.exports = nextConfig;
