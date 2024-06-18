import { createAsyncThunk, unwrapResult } from "@reduxjs/toolkit";
import { RootState } from "..";
import { WasmBucket } from "tomb_build/banyanfs";
import { destroyIsUserNew, getIsUserNew, prettyFingerprintApiKeyPem, sortByName, sortByType } from "@app/utils";
import { BrowserObject, Bucket, BucketKey } from "@app/types/bucket";
import { StorageUsageClient } from "@/api/storageUsage";
import { SnapshotsClient } from "@/api/snapshots";
import { handleNameDuplication } from "@app/utils/names";
import { updateBucketsState } from "./slice";

const storageUsageClient = new StorageUsageClient();
const snapshotsClient = new SnapshotsClient();

/** Returns list of buckets. */
export const getBuckets = createAsyncThunk(
    'getBuckets',
    async (_, { dispatch, getState }) => {
        const { tomb: { tomb } } = getState() as RootState;

        if (getIsUserNew()) {
			const bucket = unwrapResult(await dispatch(createBucketAndMount({ name: "My Drive", storageClass: 'hot', bucketType: 'interactive'})));
			destroyIsUserNew();

			return [bucket];
		};

		const wasm_buckets: WasmBucket[] = await tomb!.listBuckets();

		return wasm_buckets.map(bucket => new Bucket(
			bucket.id(),
			bucket.name(),
			null,
			bucket.bucketType(),
			bucket.storageClass(),
		));
    }
);

/** Mounts buckets, and loads info about locked state and snapshots. */
export const mountBucket = createAsyncThunk(
    'mountBucket',
    async (bucket: Bucket, { getState }) => {
        const { tomb: { tomb, encryptionKey } } = getState() as RootState;

        const mount = await tomb!.mount(bucket.id, encryptionKey!.privatePem);
		const locked = await mount.locked();
		const isSnapshotValid = await mount.hasSnapshot();

        return { id: bucket.id, mount, isSnapshotValid, locked };
    }
);

/** Creates new bucket with recieved parameters of type and storag class. */
export const createBucketAndMount = createAsyncThunk(
	'createBucketAndMount',
	async ({name, storageClass, bucketType}:{name: string, storageClass: string, bucketType: string }, { getState }) => {
		const { tomb: { tomb, encryptionKey } } = getState() as RootState;

		const { bucket: wasmBucket, mount: wasmMount } = await tomb!.createBucketAndMount(name, storageClass, bucketType, encryptionKey!.privatePem, encryptionKey!.publicPem);
		const bucket = new Bucket(
			wasmBucket.id(),
			wasmBucket.name(),
			wasmMount,
			wasmBucket.bucketType(),
			wasmBucket.storageClass(),
		);

		return bucket;
	}
);

/** Pushes keys inside of buckets list. */
export const getBucketsKeys = createAsyncThunk(
    'getBucketsKeys',
    async (_, { getState, dispatch }) => {
        const { tomb: { buckets, tomb } } = getState() as RootState;

        for (const bucket of buckets) {
			const rawKeys = await tomb!.listBucketKeys(bucket.id);
			const keys: BucketKey[] = await Promise.all(rawKeys.map(async key => {
				const fingerPrint = await prettyFingerprintApiKeyPem(key.publicKey);
				return { approved: key.approved, bucket_id: bucket.id, fingerPrint, id: key.id, pem: key.pem };
			}));

			bucket.keys = keys;
		};

		dispatch(updateBucketsState());
    }
);

/** Returns selected bucket state according to current folder location. */
export const getSelectedBucketFiles = createAsyncThunk(
    'getSelectedBucketFiles',
    async (path: string[], { getState }) => {
		const { tomb: { selectedBucket } } = getState() as RootState;
        const files = await selectedBucket!.mount!.ls(path);

		return files ? files.sort(sortByName).sort(sortByType) : [];
    }
);

/** Returns selected folder files. */
export const getExpandedFolderFiles = createAsyncThunk(
    'getExpandedFolderFiles',
    async ({folder, path}:{path: string[], folder: BrowserObject }, { getState, dispatch }) => {
        const { tomb: { selectedBucket } } = getState() as RootState;
        const files = await selectedBucket?.mount?.ls(path);
		folder.files = files ? files.sort(sortByName).sort(sortByType) : [];
    }
);


/** Returns file as ArrayBuffer */
export const getFile = createAsyncThunk(
    'getFile',
    async ({bucket, path, name}:{bucket: Bucket, path: string[], name: string }) => {
		return  await bucket.mount!.readBytes([...path, name]);
    }
);

/** Restores bucket from selected snapshot. */
export const restore = createAsyncThunk(
    'restore',
    async ({bucket, snapshotId}:{bucket: Bucket, snapshotId: string }) => {
		await snapshotsClient.restoreFromSnapshot(bucket.id, snapshotId)
    }
);

/** Generates public link to share file. */
export const shareFile = createAsyncThunk(
    'shareFile',
    async ({bucket, path}:{bucket: Bucket, path: string[] }) => {
		return await bucket.mount!.shareFile(path);
    }
);

/** Approves access key for bucket. */
export const approveBucketAccess = createAsyncThunk(
    'approveBucketAccess',
    async ({bucket, bucketKeyId}:{bucket: Bucket, bucketKeyId: string }, { dispatch }) => {
		await bucket.mount!.shareWith(bucketKeyId);
		await dispatch(getBucketsKeys());
    }
);

/** Returns list of snapshots for selected bucket. */
export const getSelectedBucketSnapshots = createAsyncThunk(
    'getBucketSnapshots',
    async (id: string) => {
		return await snapshotsClient.getSnapshots(id);
    }
);

/** Approves a new deviceKey. */
export const approveDeviceApiKey = createAsyncThunk(
    'approveDeviceApiKey',
    async (pem: string, {getState}) => {
		const { tomb: { tomb } } = getState() as RootState;
		await tomb!.approveDeviceApiKey(pem);
    }
);

/** Deletes access key for bucket. */
export const removeBucketAccess = createAsyncThunk(
    'removeBucketAccess',
    async ({ bucket, bucketKeyId }:{bucket: Bucket, bucketKeyId: string}, {getState}) => {
	/** TODO:  connect removeBucketAccess method when in will be implemented.  */

    }
);

/** Moves file/folder into different location. */
export const moveTo = createAsyncThunk(
    'moveTo',
    async ({bucket, from, to, name}:{bucket: Bucket, from: string[], to: string[], name: string }, { dispatch }) => {
		const mount = bucket.mount!;
		const extstingFiles = (await mount.ls(to)).map(file => file.name);
		const browserObjectName = handleNameDuplication(name, extstingFiles);
		await mount.mv(from, [...to, browserObjectName]);
    }
);

/** Changes name of bucket. */
export const renameBucket = createAsyncThunk(
    'renameBucket',
    async ({bucket, name}:{bucket: Bucket, name: string }, { dispatch }) => {
		await bucket.mount?.rename(name);

		return { name, bucketId: bucket.id };
    }
);

/** Creates directory inside selected bucket */
export const createDirectory = createAsyncThunk(
    'createDirectory',
    async ({bucket, path, folderLocation, name}:{bucket: Bucket, path: string[], folderLocation: string[], name: string }, { dispatch }) => {
		const mount = bucket.mount!;
		const extstingFolders = (await mount.ls(path)).map(file => file.name);

		if (extstingFolders.includes(name)) {
			throw new Error('folder already exists');
		};

		await mount.mkdir([...path, name]);
		if (path.join('') !== folderLocation.join('')) { return; }
		const files = await mount.ls(path) || [];

		return { files: files.sort(sortByName).sort(sortByType), id: bucket.id };
    }
);

/** Gets storage usage info and sets it into state. */
export const updateStorageUsageState = createAsyncThunk(
	'updateStorageUsageState',
    async () => await storageUsageClient.getStorageUsage()
);

/** Gets storage limits info and sets it into state. */
export const updateStorageLimitsState = createAsyncThunk(
	'updateStorageLimitsState',
    async () => await storageUsageClient.getStorageLimits()
);

/** Uploads file to selected bucket/directory, updates buckets state */
export const uploadFile = createAsyncThunk(
	'uploadFile',
	async ({
		bucket,
		uploadPath,
		folderLocation,
		name,
		file,
		folder
	}:
	{
		bucket: Bucket,
		uploadPath: string[],
		folderLocation: string[],
		name: string,
		file: ArrayBuffer,
		folder?: BrowserObject
	}, {getState}) => {
		const { tomb: { selectedBucket, worker } } = getState() as RootState;
		const result = {files: bucket.files, isSnapshotValid:bucket.isSnapshotValid, id: bucket.id};
		const mount = bucket.mount!;
		const extstingFiles = (await mount.ls(uploadPath)).map(file => file.name);

		let fileName = handleNameDuplication(name, extstingFiles);
		await worker?.uploadFile(bucket.id, uploadPath, fileName, file);
		if(bucket.id !== selectedBucket?.id) return result;

		const files = await mount.ls(uploadPath) || [];

		if (folder) {
			folder.files = files.sort(sortByName).sort(sortByType);

			return result;
		};

		if (uploadPath.join('') !== folderLocation.join('')) { return result; }

		const isSnapshotValid = await mount.hasSnapshot();

		return {files:  files.sort(sortByName).sort(sortByType), isSnapshotValid, id: bucket.id}
	}
);

/** Creates bucket snapshot */
export const takeColdSnapshot = createAsyncThunk(
	'takeColdSnapshot',
    async (bucket: Bucket, { getState }) => {
		const { tomb: { tomb } } = getState() as RootState;
		await bucket.mount!.snapshot();
		const snapshots = await tomb!.listBucketSnapshots(bucket.id);
		bucket.snapshots  = snapshots;
		bucket.isSnapshotValid = true;

		return { snapshots, id: bucket.id };
    }
);


/** Deletes bucket */
export const deleteBucket = createAsyncThunk(
	'deleteBucket',
    async (id: string, { dispatch, getState }) => {
		const { tomb: { tomb, selectedBucket } } = getState() as RootState;
		await tomb?.deleteBucket(id);
		await dispatch(updateStorageUsageState());
		await dispatch(getBuckets());
		if(selectedBucket?.id === id) {
			window.location.pathname = '/';
		};
    }
);


/** Deletes bucket */
export const deleteFile = createAsyncThunk(
	'deleteFile',
    async ({bucket, name, path}: {bucket: Bucket, name: string, path: string[]}, { dispatch, getState }) => {
		const mount = bucket.mount!;
		await mount.rm([...path, name]);
    }
);


