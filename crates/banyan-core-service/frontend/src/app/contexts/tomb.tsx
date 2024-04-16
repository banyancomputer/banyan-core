import React, { ReactNode, createContext, useContext, useEffect, useState } from 'react';
import { TombWasm } from 'tomb-wasm-experimental';
import { unwrapResult } from '@reduxjs/toolkit';
import { useNavigate } from 'react-router-dom';
import { wrap } from 'comlink';

import {
	BrowserObject, Bucket,
} from '@/app/types/bucket';
import { useFolderLocation } from '@app/hooks/useFolderLocation';
import { destroyIsUserNew, getIsUserNew, sortByType } from '@app/utils';
import { useAppDispatch, useAppSelector } from '@app/store';
import { BannerError, setError } from '@app/store/errors/slice';
import { getApiKey, getEncryptionKey } from '@app/store/keystore/actions';
import { StorageLimits, StorageUsage } from '@/entities/storage';
import { TombWorker } from '@/workers/tomb.worker';

interface TombInterface {
	tomb: TombWasm | null;
	buckets: Bucket[];
	storageUsage: StorageUsage;
	storageLimits: StorageLimits;
	trash: Bucket | null;
	areBucketsLoading: boolean;
	selectedBucket: Bucket | null;
	getBuckets: () => Promise<void>;
	getBucketsKeys: () => Promise<void>;
	remountBucket: (bucket: Bucket) => Promise<void>;
	selectBucket: (bucket: Bucket | null) => void;
	getSelectedBucketFiles: (path: string[]) => Promise<BrowserObject[]>;
	getFiles: (bucketId: string, path: string[]) => Promise<BrowserObject[]>;
	getExpandedFolderFiles: (path: string[], folder: BrowserObject, bucket: Bucket) => Promise<void>;
	takeColdSnapshot: (bucket: Bucket) => Promise<void>;
	getBucketSnapshots: (id: string) => Promise<any>;
	createBucketAndMount: (name: string, storageClass: string, bucketType: string) => Promise<string>;
	renameBucket: (bucket: Bucket, newName: string) => void;
	deleteBucket: (id: string) => void;
	createDirectory: (bucket: Bucket, path: string[], name: string) => Promise<void>;
	download: (bucket: Bucket, path: string[], name: string) => Promise<void>;
	getFile: (bucket: Bucket, path: string[], name: string) => Promise<any>;
	shareFile: (bucket: Bucket, path: string[]) => Promise<string>;
	makeCopy: (bucket: Bucket, path: string[], name: string) => void;
	moveTo: (bucket: Bucket, from: string[], to: string[], name: string) => Promise<void>;
	uploadFile: (nucket: Bucket, path: string[], name: string, file: any, folder?: BrowserObject) => Promise<void>;
	purgeSnapshot: (id: string) => void;
	deleteFile: (bucket: Bucket, path: string[], name: string) => void;
	approveDeviceApiKey: (pem: string) => Promise<void>;
	approveBucketAccess: (bucket: Bucket, bucketKeyId: string) => Promise<void>;
	removeBucketAccess: (bucket: Bucket, bucketKeyId: string) => Promise<void>;
	restore: (bucket: Bucket, snapshotId: string) => Promise<void>;
};

const TombContext = createContext<TombInterface>({} as TombInterface);

const worker = new Worker(new URL('../../workers/tomb.worker.ts', import.meta.url));
const tombWorker = wrap<TombWorker>(worker);

export const TombProvider = ({ children }: { children: ReactNode }) => {
	const dispatch = useAppDispatch();
	const { user } = useAppSelector(state => state.session);
	const { keystoreInitialized, escrowedKeyMaterial } = useAppSelector(state => state.keystore);
	const navigate = useNavigate();
	const [tomb, setTomb] = useState<TombWasm | null>(null);
	const [buckets, setBuckets] = useState<Bucket[]>([]);
	const [trash, setTrash] = useState<Bucket | null>(null);
	const [selectedBucket, setSelectedBucket] = useState<Bucket | null>(null);
	const [storageUsage, setStorageUsage] = useState<StorageUsage>(new StorageUsage());
	const [storageLimits, setStorageLimits] = useState<StorageLimits>(new StorageLimits());
	const [areBucketsLoading, setAreBucketsLoading] = useState<boolean>(true);
	const folderLocation = useFolderLocation();
	const { driveAlreadyExists, folderAlreadyExists } = useAppSelector(state => state.locales.messages.contexts.tomb);
	const [isWorkerReady, setIsWorkerReady] = useState(false);

	/** Returns list of buckets. */
	const getBuckets = async () => {
		const isUserNew = getIsUserNew();
		await tombWorker.getBuckets(isUserNew);
		if (isUserNew) {
			destroyIsUserNew();
		};
	};

	const remountBucket = async (bucket: Bucket) => {
		await tombWorker.remountBucket(bucket.id);
	};

	/** Pushes keys inside of buckets list. */
	const getBucketsKeys = async () => {
		await tombWorker.getBucketsKeys();
	};

	/** Returns selected bucket state according to current folder location. */
	const getSelectedBucketFiles = async (path: string[]) => {
		return await tombWorker.getSelectedBucketFiles(path)
	};
	/** Returns selected bucket state according to current folder location. */
	const getFiles = async (bucketId: string, path: string[]) => {
		return await tombWorker.getFiles(bucketId, path);
	};

	/** Returns selected folder files. */
	const getExpandedFolderFiles = async (path: string[], folder: BrowserObject, bucket: Bucket) => {
		const files = await selectedBucket?.mount?.ls(path);
		folder.files = files ? files.sort(sortByType) : [];
		setSelectedBucket(prev => ({ ...prev! }));
	};

	/** Sets selected bucket into state */
	const selectBucket = async (bucket: Bucket | null) => {
		await tombWorker.selectBucket(bucket?.id || null);
	};

	/** Creates new bucket with recieved parameters of type and storag class. */
	const createBucketAndMount = async (name: string, storageClass: string, bucketType: string): Promise<any> => {
		return await tombWorker.createBucketAndMount(name, storageClass, bucketType);
	};

	/** Returns file as ArrayBuffer */
	const getFile = async (bucket: Bucket, path: string[], name: string) => {
		return await tombWorker.getFile(bucket.id, path, name);
	};

	/** Downloads file. */
	const download = async (bucket: Bucket, path: string[], name: string) => {
		await tombWorker.download(bucket.id, path, name);
	};

	/** Creates copy of fie in same direction with "Copy of" prefix. */
	const makeCopy = async (bucket: Bucket, path: string[], name: string) => {
		await tombWorker.makeCopy(bucket.id, path, folderLocation, name);
	};

	/** Restores bucket from selected snapshot. */
	const restore = async (bucket: Bucket, snapshotId: string) => {
		await tombWorker.restore(bucket.id, snapshotId);
	};

	/** Generates public link to share file. */
	const shareFile = async (bucket: Bucket, path: string[]) => await bucket.mount!.shareFile(path);

	/** Approves access key for bucket */
	const approveBucketAccess = async (bucket: Bucket, bucketKeyId: string) => {
		await tombWorker.approveBucketAccess(bucket.id, bucketKeyId);
	};

	/** Returns list of snapshots for selected bucket */
	const getBucketSnapshots = async (id: string) => {
		return await tombWorker.getBucketSnapshots(id);
	};

	/** Approves a new deviceKey */
	const approveDeviceApiKey = async (pem: string) => {
		await tombWorker.approveDeviceApiKey(pem);
	};

	/** Deletes access key for bucket */
	const removeBucketAccess = async (bucket: Bucket, bucketKeyId: string) => {
		await tombWorker.removeBucketAccess(bucket.id, bucketKeyId);
	};

	const purgeSnapshot = async (id: string) => {
		// await tomb.purgeSnapshot(id);
	};

	/** Renames bucket */
	const moveTo = async (bucket: Bucket, from: string[], to: string[], name: string) => {
		await tombWorker.moveTo(bucket.id, from, to, name);
	};

	/** Changes name of selected bucket. */
	const renameBucket = async (bucket: Bucket, newName: string) => {
		await tombWorker.renameBucket(bucket.id, newName);
	};

	/** Creates directory inside selected bucket */
	const createDirectory = async (bucket: Bucket, path: string[], name: string) => {
		await tombWorker.createDirectory(bucket.id, path, name, folderLocation);
	};

	const updateStorageUsageState = async () => {
		await tombWorker.updateStorageUsageState();
	};

	const updateStorageLimitsState = async () => {
		await tombWorker.updateStorageLimitsState();
	};

	/** Uploads file to selected bucket/directory, updates buckets state */
	const uploadFile = async (bucket: Bucket, uploadPath: string[], name: string, file: ArrayBuffer, folder?: BrowserObject) => {
		await tombWorker.uploadFile(bucket.id, uploadPath, folderLocation, name, file);
	};

	/** Creates bucket snapshot */
	const takeColdSnapshot = async (bucket: Bucket) => {
		await tombWorker.takeColdSnapshot(bucket.id);
	};

	const deleteBucket = async (id: string) => {
		await tombWorker.deleteBucket(id);
	};

	const deleteFile = async (bucket: Bucket, path: string[], name: string) => {
		await tombWorker.deleteFile(bucket.id, path, name);
	};

	// Initialize the tomb client
	useEffect(() => {
		if (!user.id || !keystoreInitialized || !escrowedKeyMaterial) { return; }

		(async () => {
			try {
				const apiKey = unwrapResult(await dispatch(getApiKey()));
				const encryptionKey = unwrapResult(await dispatch(getEncryptionKey()));
				/** Will create tomb instance in the worker stream. */
				await tombWorker.mountTomb(apiKey, user.id, window.location.protocol + '//' + window.location.host, encryptionKey);
			} catch (error: any) {
				dispatch(setError(new BannerError(error.message)));
			}
		})();
	}, [user, keystoreInitialized, escrowedKeyMaterial]);

	useEffect(() => {
		if (tomb) {
			(async () => {
				try {
					await getBuckets();
					await updateStorageUsageState();
					await updateStorageLimitsState();
					setAreBucketsLoading(false);
				} catch (error: any) {
					dispatch(setError(new BannerError(error.message)));
					setAreBucketsLoading(false);
				}
			})();
		};
	}, [tomb]);

	useEffect(() => {
		(async () => {
			worker.onmessage = async event => {
				switch (event.data) {
					case 'tomb':
						setTomb((await tombWorker.state).tomb);
						break;
					case 'buckets':
						setBuckets((await tombWorker.state).buckets);
						setAreBucketsLoading((await tombWorker.state).areBucketsLoading);
						break;
					case 'selectedBucket':
						setSelectedBucket((await tombWorker.state).selectedBucket);
						setAreBucketsLoading((await tombWorker.state).areBucketsLoading);
						break;
					case 'configured':
						setIsWorkerReady(true);
						break;
					case 'storageUsage':
						setStorageUsage((await tombWorker.state).storageUsage);
						break;
					case 'storageLimits':
						setStorageLimits((await tombWorker.state).storageLimits);
						break;
				}
			};
		})()
	}, [])

	return (
		<TombContext.Provider
			value={{
				tomb, buckets, storageUsage, storageLimits, trash, areBucketsLoading, selectedBucket,
				getBuckets, getBucketsKeys, selectBucket, getSelectedBucketFiles, getFiles,
				takeColdSnapshot, getBucketSnapshots, createBucketAndMount, deleteBucket, remountBucket,
				getFile, renameBucket, createDirectory, uploadFile, purgeSnapshot,
				removeBucketAccess, approveBucketAccess, approveDeviceApiKey, shareFile, download, moveTo,
				restore, deleteFile, makeCopy, getExpandedFolderFiles,
			}}
		>
			{children}
		</TombContext.Provider>
	);
};

export const useTomb = () => useContext(TombContext);
