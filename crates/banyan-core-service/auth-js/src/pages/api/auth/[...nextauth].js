import NextAuth from 'next-auth';
import GoogleProvider from 'next-auth/providers/google';
import SequelizeAdapter from '@auth/sequelize-adapter';
import client, { models } from '@/lib/db/models';
import { AllowedEmailFactory } from '@/lib/db';
import { joinProviderId } from '@/lib/utils';

export const authOptions = {
	debug: process.env.NODE_ENV === 'development',
	adapter: SequelizeAdapter(client, {
		models,
		// Note: always set synchronize: false. Use the logic at:
		// @/lib/db/models/index to update the sync status.
		synchronize: false,
	}),
	providers: [
		GoogleProvider({
			clientId: process.env.GOOGLE_CLIENT_ID,
			clientSecret: process.env.GOOGLE_CLIENT_SECRET,
		}),
	],
	session: {
		// Use JSON Web Tokens for session instead of database sessions.
		strategy: 'jwt',
	},
	callbacks: {
		// Set new data in the token from the jwt callback
		async jwt({ token, account }) {
			if (account) {
				console.log('Account: ', account);
				token.providerId = joinProviderId(account.provider, account.providerAccountId);
				token.escrowedDeviceBlob = account.escrowed_device_blob;
				token.apiKeyPem = account.api_key_pem;
				token.encryptionKeyPem = account.encryption_key_pem;
			}
			return token;
		},

		async session({ session, token }) {
			session.providerId = token.providerId;
			session.escrowedDeviceBlob = token.escrowedDeviceBlob;
			session.apiKeyPem = token.apiKeyPem;
			session.encryptionKeyPem = token.encryptionKeyPem;
			return session;
		},

		async signIn({ account, profile }) {
			// Prevent sign in if the account is not allowed
			if (account.provider !== 'google') {
				return false;
			}
			const allowed = await AllowedEmailFactory.readByEmail(profile.email);
			return !!allowed;
		},
	},
};

export default NextAuth(authOptions);
