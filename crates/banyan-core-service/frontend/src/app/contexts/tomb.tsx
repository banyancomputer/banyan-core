import React, { ReactNode, createContext, useContext, useEffect, useState } from 'react';
import { TombWasm, WasmBucket, WasmMount, WasmSnapshot } from 'tomb-wasm-experimental';
import { Mutex } from 'async-mutex';
import { useModal } from '@/app/contexts/modals';

import { useKeystore } from './keystore';
import {
	BrowserObject, Bucket, BucketKey,
	BucketSnapshot, MockBucket,
} from '@/app/types/bucket';
import { useFolderLocation } from '@/app/hooks/useFolderLocation';
import { useSession } from './session';
import { prettyFingerprintApiKeyPem } from '@app/utils';

interface TombInterface {
	tomb: TombWasm | null;
	buckets: Bucket[];
	usedStorage: number;
	usageLimit: number;
	trash: Bucket;
	isTrashLoading: boolean;
	areBucketsLoading: boolean;
	selectedBucket: Bucket | null;
	getBuckets: () => Promise<void>;
	getBucketsFiles: () => Promise<void>;
	getBucketsKeys: () => Promise<void>;
	selectBucket: (bucket: Bucket | null) => void;
	getSelectedBucketFiles: (path: string[]) => void;
	getExpandedFolderFiles: (path: string[], folder: BrowserObject, bucket: Bucket) => Promise<void>;
	takeColdSnapshot: (bucket: Bucket) => Promise<void>;
	getBucketShapshots: (id: string) => Promise<BucketSnapshot[]>;
	createBucket: (name: string, storageClass: string, bucketType: string) => Promise<void>;
	deleteBucket: (id: string) => void;
	getTrashBucket: () => Promise<void>;
	createDirectory: (bucket: Bucket, path: string[], name: string) => Promise<void>;
	download: (bucket: Bucket, path: string[], name: string) => Promise<void>;
	getFile: (bucket: Bucket, path: string[], name: string) => Promise<ArrayBuffer>;
	shareFile: (bucket: Bucket, file: BrowserObject) => Promise<string>;
	makeCopy: (bucket: Bucket, path: string[], name: string) => void;
	moveTo: (bucket: Bucket, from: string[], to: string[]) => Promise<void>;
	uploadFile: (nucket: Bucket, path: string[], name: string, file: any, folder?: BrowserObject) => Promise<void>;
	purgeSnapshot: (id: string) => void;
	deleteFile: (bucket: Bucket, path: string[], name: string) => void;
	completeDeviceKeyRegistration: (fingerprint: string) => Promise<void>;
	approveBucketAccess: (bucket: Bucket, bucket_key_id: string) => Promise<void>;
	removeBucketAccess: (id: string) => Promise<void>;
	restore: (bucket: Bucket, snapshot: WasmSnapshot) => Promise<void>;
};

type TombBucket = Bucket & { mount: WasmMount };
const mutex = new Mutex();

const TombContext = createContext<TombInterface>({} as TombInterface);

export const TombProvider = ({ children }: { children: ReactNode }) => {
	/** TODO: rework session logic. */
	const { userData } = useSession();
	const { openEscrowModal } = useModal();
	const { isLoading, keystoreInitialized, getEncryptionKey, getApiKey, escrowedKeyMaterial } = useKeystore();
	const [tomb, setTomb] = useState<TombWasm | null>(null);
	const [buckets, setBuckets] = useState<TombBucket[]>([]);
	const [trash, setTrash] = useState<TombBucket>(new MockBucket());
	const [selectedBucket, setSelectedBucket] = useState<Bucket | null>(null);
	const [usedStorage, setUsedStorage] = useState<number>(0);
	const [usageLimit, setUsageLimit] = useState<number>(0);
	const [isTrashLoading, setIsTrashLoading] = useState<boolean>(true);
	const [areBucketsLoading, setAreBucketsLoading] = useState<boolean>(false);
	const folderLocation = useFolderLocation();

	/** Prevents rust recursion error. */
	const tombMutex = async <T,>(tomb: T, callback: (tomb: T) => Promise<any>) => {
		const release = await mutex.acquire();
		try {
			return await callback(tomb);
		} catch (err) {
			console.error('tombMutex', err);
		} finally {
			release();
		}
	};

	/** Returns list of buckets. */
	const getBuckets = async () => {
		tombMutex(tomb, async tomb => {
			const wasm_buckets: WasmBucket[] = await tomb!.listBuckets();
			setBuckets(wasm_buckets.map(bucket => ({
				mount: {} as WasmMount,
				id: bucket.id(),
				name: bucket.name(),
				storageClass: bucket.storageClass(),
				bucketType: bucket.bucketType(),
				files: [],
				snapshots: [],
				keys: [],
			})));
		});
	};

	const getBucketsFiles = async () => {
		setAreBucketsLoading(true);
		tombMutex(tomb, async tomb => {
			const key = await getEncryptionKey();
			const wasm_bukets: Bucket[] = [];
			for (const bucket of buckets) {
				const mount = await tomb!.mount(bucket.id, key.privatePem);
				const files: BrowserObject[] = await mount.ls([]) || [];
				const snapshots = await tomb!.listBucketSnapshots(bucket.id);
				wasm_bukets.push({
					...bucket,
					mount,
					snapshots,
					files,
				});
			};
			setBuckets(wasm_bukets);
			setAreBucketsLoading(false);
		});
	};

	const getBucketsKeys = async () => {
		setAreBucketsLoading(true);
		tombMutex(tomb, async tomb => {
			const key = await getEncryptionKey();
			const wasm_bukets: Bucket[] = [];
			for (const bucket of buckets) {
				const mount = await tomb!.mount(bucket.id, key.privatePem);
				const rawKeys = await tomb!.listBucketKeys(bucket.id);
				const keys: BucketKey[] = [];
				for (let key of rawKeys) {
					const pem = key.pem();
					const approved = key.approved();
					const id = key.id();
					const fingerPrint = await prettyFingerprintApiKeyPem(pem);
					keys.push({ approved, bucket_id: bucket.id, fingerPrint, id, pem })
				};
				wasm_bukets.push({
					...bucket,
					mount,
					keys,
				});
			}
			setBuckets(wasm_bukets);
			setAreBucketsLoading(false);
		});
	};

	/** Returns selected bucket state according to current folder location. */
	const getSelectedBucketFiles = async (path: string[]) => {
		tombMutex(selectedBucket!.mount, async mount => {
			setAreBucketsLoading(true);
			const files = await mount.ls(path);
			await setSelectedBucket(bucket => bucket ? { ...bucket, files } : bucket);
			setAreBucketsLoading(false);
		});
	};
	/** Returns selected folder files. */
	const getExpandedFolderFiles = async (path: string[], folder: BrowserObject, bucket: Bucket) => {
		await tombMutex(selectedBucket!.mount, async mount => {
			const files = await mount.ls(path);
			folder.files = files;
			selectBucket({ ...bucket });
		});
	};

	/** Sets selected bucket into state */
	const selectBucket = async (bucket: Bucket | null) => {
		if (!bucket) {
			setSelectedBucket(null);

			return;
		};

		const key = await getEncryptionKey();
		await tombMutex(tomb, async tomb => {
			const mount = await tomb!.mount(bucket?.id, key.privatePem);
			setSelectedBucket({ ...bucket, mount });
		});
	};

	/** Creates new bucket with recieved parameters of type and storag class. */
	const createBucket = async (name: string, storageClass: string, bucketType: string) => {
		await tombMutex(tomb, async tomb => {
			const key = await getEncryptionKey();
			const wasmBucket = await tomb!.createBucket(name, storageClass, bucketType, key.publicPem);
			const mount = await tomb!.mount(wasmBucket.id(), key.privatePem);
			const files = await mount.ls([]);
			const snapshots = await tomb!.listBucketSnapshots(wasmBucket.id());
			const bucket = {
				mount,
				id: wasmBucket.id(),
				name: wasmBucket.name(),
				storageClass: wasmBucket.storageClass(),
				bucketType: wasmBucket.bucketType(),
				files: files || [],
				snapshots,
				keys: [],
			};

			setBuckets(prev => [...prev, bucket]);
		});
	};

	/** Returns file as ArrayBuffer */
	const getFile = async (bucket: Bucket, path: string[], name: string) => await tombMutex(bucket.mount, async mount => await mount!.readBytes([...path, name]));

	/** Downloads file. */
	const download = async (bucket: Bucket, path: string[], name: string) => {
		const link = document.createElement('a');
		const arrayBuffer: Uint8Array = await getFile(bucket, path, name);
		const blob = new Blob([arrayBuffer], { type: 'application/octet-stream' });
		const objectURL = URL.createObjectURL(blob);
		link.href = objectURL;
		link.download = name;
		document.body.appendChild(link);
		link.click();
	};

	/** Creates copy of fie in same direction with "Copy of" prefix. */
	const makeCopy = async (bucket: Bucket, path: string[], name: string) => {
		const arrayBuffer: ArrayBuffer = await getFile(bucket, path, name);
		await uploadFile(bucket, path, `Copy of ${name}`, arrayBuffer);
	};

	/** Retuns array buffer of selected file. */
	const restore = async (bucket: Bucket, snapshot: WasmSnapshot) => await tombMutex(bucket.mount, async mount => await mount.restore(snapshot));

	/** Generates public link to share file. */
	const shareFile = async (bucket: Bucket, file: BrowserObject) =>
		/** TODO: implement sharing logic when it will be added to tomb. */
		''
		;

	/** Approves access key for bucket */
	const approveBucketAccess = async (bucket: Bucket, bucket_key_id: string) => {
		await tombMutex(bucket.mount, async mount => {
			await mount.shareWith(bucket_key_id);
		});
	};

	/** Returns list of snapshots for selected bucket */
	const getBucketShapshots = async (id: string) => await tombMutex(tomb, async tomb => await tomb!.listBucketSnapshots(id));

	/** Approves a new deviceKey */
	const completeDeviceKeyRegistration = async (fingerprint: string) => await tombMutex(tomb, async tomb => await tomb!.completeDeviceKeyRegistration(fingerprint));


	/** Deletes access key for bucket */
	const removeBucketAccess = async (id: string) => {
		/** TODO:  connect removeBucketAccess method when in will be implemented.  */
	};

	const purgeSnapshot = async (id: string) => {
		// await tomb.purgeSnapshot(id);
	};


	/** Renames bucket */
	const moveTo = async (bucket: Bucket, from: string[], to: string[]) => {
		await tombMutex(bucket.mount, async mount => {
			await mount.mv(from, to);
		});
	};

	/** Internal function which looking for selected bucket and updates it, or bucket in buckets list if no bucket selected. */
	const updateBucketsState = (key: 'keys' | 'files' | 'snapshots', elements: BrowserObject[] | BucketSnapshot[], id: string,) => {
		/** If we are on buckets list screen there is no selected bucket in state. */
		if (selectedBucket?.id === id) {
			setSelectedBucket(bucket => bucket ? { ...bucket, [key]: elements } : bucket);

			return;
		};

		setBuckets(buckets => buckets.map(bucket => {
			if (bucket.id === id) {
				return { ...bucket, [key]: elements };
			}

			return bucket;
		}));
	};

	/** Creates directory inside selected bucket */
	const createDirectory = async (bucket: Bucket, path: string[], name: string) => {
		await tombMutex(bucket.mount, async mount => {
			await mount.mkdir([...path, name]);
			if (path.join('') !== folderLocation.join('')) { return; }
			const files = await mount.ls(path) || [];
			await updateBucketsState('files', files, bucket.id);
		});
	};

	const getStorageUsageState = async () => {
		/** Returns used storage amount in bytes */
		await tombMutex(tomb, async tomb => {
			const usedStorage = +(await tomb!.getUsage()).toString();
			const usageLimit = +(await tomb!.getUsageLimit()).toString();
			await setUsedStorage(usedStorage);
			await setUsageLimit(usageLimit);
		});
	};

	/** Uploads file to selected bucket/directory, updates buckets state */
	const uploadFile = async (bucket: Bucket, uploadPath: string[], name: string, file: ArrayBuffer, folder?: BrowserObject) => {
		try {
			tombMutex(bucket.mount, async mount => {
				await mount.write([...uploadPath, name], file);
				if (folder) {
					const files = await mount.ls(uploadPath);
					folder.files = files;
					selectBucket({ ...bucket });

					return;
				}
				if (uploadPath.join('') !== folderLocation.join('')) { return; }
				const files = await mount.ls(uploadPath) || [];
				await updateBucketsState('files', files, bucket.id);
			});
			await getStorageUsageState();
		} catch (error: any) {
			console.log('uploadError', error);
		}
	};

	/** Creates bucket snapshot */
	const takeColdSnapshot = async (bucket: Bucket) => {
		await tombMutex(bucket.mount, async mount => {
			await mount.snapshot();
		});
		await tombMutex(tomb, async tomb => {
			const snapshots = await tomb!.listBucketSnapshots(bucket.id);
			await updateBucketsState('snapshots', snapshots, bucket.id);
		});
	};

	/** TODO: implement after adding to tomb-wasm */
	const getTrashBucket: () => Promise<void> = async () => {
		// setIsTrashLoading(true);
		// const trash = await tomb();
		// const files = await getFiles(trash.id, '/');
		// setTrash({ ...trash, files });
		// setIsTrashLoading(false);
	};


	const deleteBucket = async (id: string) => {
		await tomb?.deleteBucket(id);
		await getBuckets();
		await getStorageUsageState();
	};

	const deleteFile = async (bucket: Bucket, path: string[], name: string) => {
		await tombMutex(bucket.mount, async mount => {
			await mount.rm([...path, name]);
		});
	};

	// Initialize the tomb client
	useEffect(() => {
		if (!userData || !keystoreInitialized) { return; }

		(async () => {
			try {
				const apiKey = await getApiKey();
				const TombWasm = (await import('tomb-wasm-experimental')).TombWasm;
				const tomb = new TombWasm(
					apiKey.privatePem,
					userData.user.id,
					process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001',
					process.env.NEXT_PUBLIC_DATA_URL || 'http://localhost:3002',
				);
				setTomb(await tomb);
			} catch (err) {
				console.error(err);
			}
		})();
	}, [userData, keystoreInitialized, isLoading, escrowedKeyMaterial]);

	useEffect(() => {
		if (!keystoreInitialized && !isLoading) {
			openEscrowModal(!!escrowedKeyMaterial);
		};
	}, [isLoading, keystoreInitialized]);

	useEffect(() => {
		if (tomb) {
			(async () => {
				try {
					await getBuckets();
					await getStorageUsageState();
				} catch (error: any) { };
			})();
		};
	}, [tomb]);

	return (
		<TombContext.Provider
			value={{
				tomb, buckets, trash, usedStorage, usageLimit, areBucketsLoading, isTrashLoading, selectedBucket,
				getBuckets, getBucketsFiles, getBucketsKeys, selectBucket, getSelectedBucketFiles,
				takeColdSnapshot, getBucketShapshots, createBucket, deleteBucket, getTrashBucket,
				getFile, createDirectory, uploadFile, purgeSnapshot,
				removeBucketAccess, approveBucketAccess, completeDeviceKeyRegistration, shareFile, download, moveTo,
				restore, deleteFile, makeCopy, getExpandedFolderFiles,
			}}
		>
			{children}
		</TombContext.Provider>
	);
};

export const useTomb = () => useContext(TombContext);