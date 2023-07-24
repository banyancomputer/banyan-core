import NextAuth from 'next-auth';
import GoogleProvider from 'next-auth/providers/google';
import { TypeORMAdapter } from '@auth/typeorm-adapter';
import { pgConnection, AllowedFactory } from '@/lib/db';

export const authOptions = {
	debug: process.env.NODE_ENV === 'development',
	adapter: TypeORMAdapter({
		sychronize: process.env.NODE_ENV === 'development',
		...pgConnection,
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
			// console.log(
			// 	'JWT callback: ',
			// 	'\nToken -> ',
			// 	token,
			// 	'\nAccount -> ',
			// 	account
			// );

			if (account) {
				token.accessToken = account.access_token;
				token.provider = account.provider;
				token.id = account.providerAccountId;
			}
			return token;
		},

		// // Set new data in the session from the user object, using token modified in jwt callback
		async session({ session, token }) {
			session.accessToken = token.accessToken;
			session.id = token.id;
			session.provider = token.provider;
			return session;
		},

		async signIn({ account, profile }) {
			// Prevent sign in if the account is not allowed
			if (account.provider !== 'google') {
				return false;
			}
			// console.log(
			// 	'Sign in callback: ',
			// 	'\nAccount -> ',
			// 	account,
			// 	'\nProfile -> ',
			// 	profile
			// );
			const allowed = await AllowedFactory.readByEmail(profile.email);
			return !!allowed;
		},
	},
};

export default NextAuth(authOptions);
