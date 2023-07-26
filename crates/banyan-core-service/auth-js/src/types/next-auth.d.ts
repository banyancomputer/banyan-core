import NextAuth from 'next-auth';

// Note: see https://next-auth.js.org/getting-started/typescript#extend-default-interface-properties for more info on module augmentation in NextAuth
declare module 'next-auth' {
	// /**
	//  * Returned by `useSession`, `getSession` and received as a prop on the `SessionProvider` React Context
	//  */
	interface Session {
		// The user's id in the database
		userId: string;
		// The provider used to sign in
		provider: string;
		// The provider's unique identifier for the user
		providerAccountId: string;
	}
}
