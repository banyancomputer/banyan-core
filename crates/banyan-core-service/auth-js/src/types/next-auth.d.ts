import NextAuth from 'next-auth';

// Note: see https://next-auth.js.org/getting-started/typescript#extend-default-interface-properties for more info on module augmentation in NextAuth
declare module 'next-auth' {
	// /**
	//  * Returned by `useSession`, `getSession` and received as a prop on the `SessionProvider` React Context
	//  */
	interface Session {
		// The Account's provider identifier (this is just <provider>:<providerAccountId>)
		// You can use this to search for an account by provider and providerAccountId
		// See lib/db/account.ts for more details
		providerId: string;
		// // The user's private key material escrowed to our Auth Service
		// escrowedDeviceBlob: string | null;
		// // The user's public key material for escrowed ecdh private key material
		// apiKeyPem: string | null;
		// // The user's public key material for escrowed ecdsa private key material
		// encryptionKeyPem: string | null;
	}
}
