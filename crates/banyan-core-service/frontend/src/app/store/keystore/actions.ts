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
		const keyMaterial = await keystore!.genKeyMaterial();
		const privateKeyMaterial = await keystore!.exportPrivateKeyMaterial(keyMaterial);
		const escrowedKeyMaterial = await keystore!.escrowKeyMaterial(keyMaterial, passphrase);

		await authClient.escrowDevice(escrowedKeyMaterial);
		dispatch(setEscrowedKeyMaterial(escrowedKeyMaterial));

		return privateKeyMaterial;
	});

	// Recovers a device's private key material from escrow
	export const recoverDevice = createAsyncThunk(
        'recoverDevice',
        async (passphrase: string, { getState }) => {
        const {keystore: {escrowedKeyMaterial, keystore}} = getState() as RootState;

		return await keystore!.recoverKeyMaterial(
			escrowedKeyMaterial!,
			passphrase
		);
	});

    export const getEscrowedKeyMaterial = createAsyncThunk(
        'getEscrowedKeyMaterial',
        async () => await userClient.getEscrowedKeyMaterial());

	// Initialize a keystore based on the user's passphrase
	export const initializeKeystore = createAsyncThunk(
        'initializeKeystore',
    async (passkey: string, { getState, dispatch }) => {
        const {keystore: {escrowedKeyMaterial, keystore}} = getState() as RootState;
		let privateKeyMaterial: PrivateKeyMaterial;

		if (!keystore) {
			throw new Error('No keystore');
		};

		try {
			if (escrowedKeyMaterial) {
				privateKeyMaterial = unwrapResult(await dispatch(recoverDevice(passkey)));
			} else {
				privateKeyMaterial = unwrapResult(await dispatch(escrowDevice(passkey)));
			}
			let localKey = getLocalKey();
			// Cache the key material encrypted with the session key
			await keystore!.cachePrivateKeyMaterial(
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
		await keystore?.clear();
		// Purge the local key cookie
		destroyLocalKey();
	});


	// Get the user's Encryption Key Pair as a Public / Private PEM combo
	export const getEncryptionKey = createAsyncThunk(
        'getEncryptionKey',
        async (_, { getState }): Promise<{ privatePem: string, publicPem: string }> => {
        const {keystore: {escrowedKeyMaterial, keystore}} = getState() as RootState;
		const localKey = getLocalKey();
		const keyMaterial = await keystore!.retrieveCachedPrivateKeyMaterial(localKey.key, localKey.id);

		return {
			privatePem: keyMaterial.encryptionPrivateKeyPem,
			publicPem: escrowedKeyMaterial!.encryptionPublicKeyPem
		};
	});

	// Get the user's API Key as a Private / Public PEM combo
	export const getApiKey = createAsyncThunk(
        'getApiKey',
        async (_, {getState}): Promise<{ privatePem: string, publicPem: string }> => {
        const {keystore: { escrowedKeyMaterial, keystore }} = getState() as RootState;

		const localKey = getLocalKey();
		const privateKeyMaterial = await keystore!.retrieveCachedPrivateKeyMaterial(localKey.key, localKey.id);

		return {
			privatePem: privateKeyMaterial.apiPrivateKeyPem,
			publicPem: escrowedKeyMaterial!.apiPublicKeyPem
		};
	});