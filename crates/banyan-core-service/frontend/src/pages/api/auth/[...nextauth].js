import NextAuth from 'next-auth';
import GoogleProvider from 'next-auth/providers/google';
import SequelizeAdapter from '@auth/sequelize-adapter';
import client, { models } from '@/lib/db/models';
import { AccountFactory, AllowedEmailFactory } from '@/lib/db';
import { joinProviderId } from '@/utils';

export const authOptions = {
    debug: process.env.NODE_ENV === 'development',
    adapter: SequelizeAdapter(client, {
        models,
        // Note: always set synchronize: false.
        // Rely on sqlx to handle migrations.
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
        // TODO: Do we even need the sessions table?
        strategy: 'jwt',
    },
    callbacks: {
        // Set new data in the token from the jwt callback
        async jwt({ token, account }) {
            if (account) {
                console.log('Account: ', account);
                token.providerId = joinProviderId(
                    account.provider,
                    account.providerAccountId
                );
            }

            return token;
        },

        async session({ session, token }) {
            session.providerId = token.providerId;
			session.accountId = await AccountFactory.idFromProviderId(
				token.providerId
			);

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
