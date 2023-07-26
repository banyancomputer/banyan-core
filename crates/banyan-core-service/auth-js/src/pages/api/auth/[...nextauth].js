import NextAuth from 'next-auth';
import GoogleProvider from 'next-auth/providers/google';
import { TypeORMAdapter } from '@auth/typeorm-adapter';
import { connection, AllowedEmailFactory } from '@/lib/db';

export const authOptions = {
	debug: process.env.NODE_ENV === 'development',
	adapter: TypeORMAdapter(connection),
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
				token.provider = account.provider;
				token.providerAccountId = account.providerAccountId;
			}
			return token;
		},

		async session({ session, token }) {
			session.userId = `${token.provider}-${token.providerAccountId}`;
			session.provider = token.provider;
			session.providerAccountId = token.providerAccountId;
			return session;
		},

		async signIn({ account, profile }) {
			// Prevent sign in if the account is not allowed
			if (account.provider !== 'google') {
				return false;
			}
			console.log(
				'Sign in callback: ',
				'\nAccount -> ',
				account,
				'\nProfile -> ',
				profile
			);
			const allowed = await AllowedEmailFactory.readByEmail(profile.email);
			return !!allowed;
		},
	},
};

export default NextAuth(authOptions);
