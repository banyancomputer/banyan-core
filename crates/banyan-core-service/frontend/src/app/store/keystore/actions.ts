import { createAsyncThunk, unwrapResult } from "@reduxjs/toolkit";

import { UserClient } from "@/api/user";
import { AuthClient } from "@/api/auth";
import { RootState } from "..";
import { PrivateKeyMaterial } from "@app/utils/crypto/types";
import { setEscrowedKeyMaterial } from "@app/store/keystore/slice";
import { destroyLocalKey, getLocalKey } from "@app/utils";

const userClient = new UserClient();
const authClient = new AuthClient();

	// Register a new user in firestore
	export const escrowDevice = createAsyncThunk(
        'escrowDevice',
        async (passphrase: string, { dispatch, getState } ): Promise<PrivateKeyMaterial> => {
        const {keystore: {keystore}} = getState() as RootState;

		if (!keystore) {
			throw new Error('No keystore');
		};

		const keyMaterial = await keystore.genKeyMaterial();
		const privateKeyMaterial = await keystore.exportPrivateKeyMaterial(keyMaterial);
		const escrowedKeyMaterial = await keystore.escrowKeyMaterial(
			keyMaterial,
			passphrase
		);
		// Escrow the user's private key material
		await authClient
			.escrowDevice(escrowedKeyMaterial)
			.then(() => {
				// Set the escrowed key material in the context state and cookies
				dispatch(setEscrowedKeyMaterial(escrowedKeyMaterial))
			})
			.catch((err) => {
				throw new Error("Error escrowing device: " + err.message);
			});
		return privateKeyMaterial;
	});

	// Recovers a device's private key material from escrow
	export const recoverDevice = createAsyncThunk(
        'recoverDevice',
        async (passphrase: string, { getState }) => {
        const {keystore: {escrowedKeyMaterial, keystore}} = getState() as RootState;

		if (!keystore) {
			throw new Error('No keystore');
		}
		if (!escrowedKeyMaterial) {
			throw new Error('No escrowed device');
		}
		return await keystore.recoverKeyMaterial(
			escrowedKeyMaterial,
			passphrase
		);
	});

    export const getEscrowedKeyMaterial = createAsyncThunk(
        'getEscrowedKeyMaterial',
        async () => {
        return await userClient.getEscrowedKeyMaterial();
    });

	// Initialize a keystore based on the user's passphrase
	export const initializeKeystore = createAsyncThunk(
        'initializeKeystore',
    async (passkey: string, { getState, dispatch }) => {
        const {keystore: {escrowedKeyMaterial, keystore}} = getState() as RootState;
		let privateKeyMaterial: PrivateKeyMaterial;
		// TODO: better error handling
		if (!keystore) {
			throw new Error('Keystore not initialized');
		};

		try {
			if (escrowedKeyMaterial) {
				privateKeyMaterial = unwrapResult(await dispatch(recoverDevice(passkey)));
			} else {
				privateKeyMaterial = unwrapResult(await dispatch(escrowDevice(passkey)));
			}
			let localKey = getLocalKey();
			// Cache the key material encrypted with the session key
			await keystore.cachePrivateKeyMaterial(
				privateKeyMaterial,
				localKey.key,
				localKey.id
			);
		} catch (err: any) {
			throw new Error(err.message);
		}
	});

    export const purgeKeystore = createAsyncThunk(
        'purgeKeystore',
        async (_, {getState}): Promise<void> => {
        const {keystore: {keystore}} = getState() as RootState;
		if (keystore) {
			await keystore.clear();
		}
		// Purge the local key cookie
		destroyLocalKey();
	});


	// Get the user's Encryption Key Pair as a Public / Private PEM combo
	export const getEncryptionKey = createAsyncThunk(
        'getEncryptionKey',
        async (_, { getState }): Promise<{ privatePem: string, publicPem: string }> => {
        const {keystore: {escrowedKeyMaterial, keystore, keystoreInitialized}} = getState() as RootState;

		if (!keystore) {
			throw new Error('No keystore');
		};
		if (!keystoreInitialized) {
			throw new Error('Keystore not initialized');
		};
		if (!escrowedKeyMaterial) {
			throw new Error('Missing escrowed data');
		};
		let localKey = getLocalKey();
		const keyMaterial = await keystore.retrieveCachedPrivateKeyMaterial(
			localKey.key, localKey.id
		);

		return {
			privatePem: keyMaterial.encryptionPrivateKeyPem,
			publicPem: escrowedKeyMaterial.encryptionPublicKeyPem
		};
	});

	// Get the user's API Key as a Private / Public PEM combo
	export const getApiKey = createAsyncThunk(
        'getApiKey',
        async (_, {getState}): Promise<{ privatePem: string, publicPem: string }> => {
        const {keystore: { escrowedKeyMaterial, keystore, keystoreInitialized }} = getState() as RootState;
		if (!keystore || !keystoreInitialized) {
			throw new Error('Keystore not initialized');
		};
		if (!escrowedKeyMaterial) {
			throw new Error('Missing escrowed data');
		};
		let localKey = getLocalKey();
		const privateKeyMaterial = await keystore.retrieveCachedPrivateKeyMaterial(
			localKey.key, localKey.id
		);

		return {
			privatePem: privateKeyMaterial.apiPrivateKeyPem,
			publicPem: escrowedKeyMaterial.apiPublicKeyPem
		};
	});