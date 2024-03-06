import React, { ReactNode, createContext, useContext, useEffect, useState } from 'react';
import { TombWasm, WasmBucket, WasmMount, WasmSnapshot } from 'tomb-wasm-experimental';
import { Mutex } from 'async-mutex';
import { useNavigate } from 'react-router-dom';

import { TermsAndConditionsModal } from '@components/common/Modal/TermsAndConditionsModal';
import { TermaAndConditions } from '@components/common/TermsAndConditions';

import { useModal } from '@/app/contexts/modals';
import { useKeystore } from './keystore';
import {
	BrowserObject, Bucket, BucketKey,
	BucketSnapshot,
} from '@/app/types/bucket';
import { useFolderLocation } from '@/app/hooks/useFolderLocation';
import { destroyIsUserNew, getIsUserNew, prettyFingerprintApiKeyPem, sortByType } from '@app/utils';
import { TermsAndColditionsClient } from '@/api/termsAndConditions';
import { UserClient } from '@/api/user';
import { handleNameDuplication } from '@utils/names';
import { StorageUsageClient } from '@/api/storageUsage';
import { useAppDispatch, useAppSelector } from '../store';
import { BannerError, setError } from '../store/errors/slice';

interface TombInterface {
	tomb: TombWasm | null;
	buckets: Bucket[];
	storageUsage: { usage: number, softLimit: number, hardLimit: number };
	trash: Bucket | null;
	areBucketsLoading: boolean;
	selectedBucket: Bucket | null;
	getBuckets: () => Promise<void>;
	getBucketsFiles: () => Promise<void>;
	getBucketsKeys: () => Promise<void>;
	remountBucket: (bucket: Bucket) => Promise<void>;
	selectBucket: (bucket: Bucket | null) => void;
	getSelectedBucketFiles: (path: string[]) => void;
	getExpandedFolderFiles: (path: string[], folder: BrowserObject, bucket: Bucket) => Promise<void>;
	takeColdSnapshot: (bucket: Bucket) => Promise<void>;
	getBucketShapshots: (id: string) => Promise<BucketSnapshot[]>;
	createBucketAndMount: (name: string, storageClass: string, bucketType: string) => Promise<string>;
	renameBucket: (bucket: Bucket, newName: string) => void;
	deleteBucket: (id: string) => void;
	createDirectory: (bucket: Bucket, path: string[], name: string) => Promise<void>;
	download: (bucket: Bucket, path: string[], name: string) => Promise<void>;
	getFile: (bucket: Bucket, path: string[], name: string) => Promise<ArrayBuffer>;
	shareFile: (bucket: Bucket, path: string[]) => Promise<string>;
	makeCopy: (bucket: Bucket, path: string[], name: string) => void;
	moveTo: (bucket: Bucket, from: string[], to: string[], name: string) => Promise<void>;
	uploadFile: (nucket: Bucket, path: string[], name: string, file: any, folder?: BrowserObject) => Promise<void>;
	purgeSnapshot: (id: string) => void;
	deleteFile: (bucket: Bucket, path: string[], name: string) => void;
	approveDeviceApiKey: (pem: string) => Promise<void>;
	approveBucketAccess: (bucket: Bucket, bucket_key_id: string) => Promise<void>;
	removeBucketAccess: (id: string) => Promise<void>;
	restore: (bucket: Bucket, snapshot: WasmSnapshot) => Promise<void>;
};
const storageUsageClient = new StorageUsageClient();

const mutex = new Mutex();

const TombContext = createContext<TombInterface>({} as TombInterface);

export const TombProvider = ({ children }: { children: ReactNode }) => {
	const dispatch = useAppDispatch();
	const { user, escrowedKeyMaterial } = useAppSelector(state => state.session);
	const navigate = useNavigate();
	const { openEscrowModal, openModal } = useModal();
	const { isLoading, keystoreInitialized, getEncryptionKey, getApiKey, isLoggingOut } = useKeystore();
	const [tomb, setTomb] = useState<TombWasm | null>(null);
	const [buckets, setBuckets] = useState<Bucket[]>([]);
	const [trash, setTrash] = useState<Bucket | null>(null);
	const [areTermsAccepted, setAreTermsAccepted] = useState(false);
	const [selectedBucket, setSelectedBucket] = useState<Bucket | null>(null);
	const [storageUsage, setStorageUsage] = useState<{ usage: number, softLimit: number, hardLimit: number }>({ usage: 0, softLimit: 0, hardLimit: 0 });
	const [areBucketsLoading, setAreBucketsLoading] = useState<boolean>(true);
	const folderLocation = useFolderLocation();

	/** Prevents rust recursion error. */
	const tombMutex = async <T,>(tomb: T, callback: (tomb: T) => Promise<any>) => {
		const release = await mutex.acquire();
		try {
			return await callback(tomb);
		} catch (error: any) {
			throw new Error(error);
		} finally {
			release();
		}
	};

	/** Returns list of buckets. */
	const getBuckets = async () => {
		return await tombMutex(tomb, async tomb => {
			const key = await getEncryptionKey();
			const wasm_buckets: WasmBucket[] = await tomb!.listBuckets();
			console.log(getIsUserNew());

			if (getIsUserNew()) {
				createBucketAndMount("My Drive", 'hot', 'interactive');
				destroyIsUserNew();
				return;
			}
			const buckets: Bucket[] = [];
			for (let bucket of wasm_buckets) {
				let mount;
				let locked;
				let isSnapshotValid;
				try {
					mount = await tomb!.mount(bucket.id(), key.privatePem);
					locked = await mount.locked();
					isSnapshotValid = await mount.hasSnapshot();
				} catch (error: any) { }
				buckets.push({
					mount: mount || null,
					id: bucket.id(),
					name: bucket.name(),
					storageClass: bucket.storageClass(),
					bucketType: bucket.bucketType(),
					files: [],
					snapshots: [],
					keys: [],
					locked: locked || false,
					isSnapshotValid: isSnapshotValid || false
				});
			};

			setBuckets(buckets);
		});
	};

	/** Pushes files and snapshots inside of buckets list. */
	const getBucketsFiles = async () => {
		return await tombMutex(tomb, async tomb => {
			const wasm_bukets: Bucket[] = [];
			for (const bucket of buckets) {
				const files: BrowserObject[] = bucket.mount ? await bucket.mount!.ls([]) : [];
				const snapshots = await tomb!.listBucketSnapshots(bucket.id);
				wasm_bukets.push({
					...bucket,
					snapshots,
					files,
				});
			};
			setBuckets(wasm_bukets);
			setTimeout(() => {
				setAreBucketsLoading(false);
			}, 300);
		});
	};

	const remountBucket = async (bucket: Bucket) => {
		return await tombMutex(tomb, async tomb => {
			const key = await getEncryptionKey();
			const mount = await tomb!.mount(bucket.id, key.privatePem);
			const locked = await mount.locked();
			const isSnapshotValid = await mount.hasSnapshot();
			setBuckets(prev => prev.map(element => element.id === bucket.id ? { ...element, mount, locked, isSnapshotValid } : element));
		});
	};

	/** Pushes keys inside of buckets list. */
	const getBucketsKeys = async () => {
		setAreBucketsLoading(true);
		return await tombMutex(tomb, async tomb => {
			const wasm_bukets: Bucket[] = [];
			for (const bucket of buckets) {
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
					keys,
				});
			}
			setBuckets(wasm_bukets);
			setAreBucketsLoading(false);
		});
	};

	/** Returns selected bucket state according to current folder location. */
	const getSelectedBucketFiles = async (path: string[]) => {
		return await tombMutex(selectedBucket!.mount!, async mount => {
			setAreBucketsLoading(true);
			const files = await mount.ls(path);
			await setSelectedBucket(bucket => ({ ...bucket!, files: files.sort(sortByType) }));
			setAreBucketsLoading(false);
		});
	};

	/** Returns selected folder files. */
	const getExpandedFolderFiles = async (path: string[], folder: BrowserObject, bucket: Bucket) => {
		return await tombMutex(selectedBucket!.mount!, async mount => {
			const files = await mount.ls(path);
			folder.files = files.sort(sortByType);
			setSelectedBucket(prev => ({ ...prev! }));
		});
	};

	/** Sets selected bucket into state */
	const selectBucket = async (bucket: Bucket | null) => {
		setSelectedBucket(bucket);
	};

	/** Creates new bucket with recieved parameters of type and storag class. */
	const createBucketAndMount = async (name: string, storageClass: string, bucketType: string): Promise<string> => {
		return await tombMutex(tomb, async tomb => {
			const key = await getEncryptionKey();
			const { bucket: wasmBucket, mount: wasmMount } = await tomb!.createBucketAndMount(name, storageClass, bucketType, key.privatePem, key.publicPem);
			const bucket = {
				mount: wasmMount,
				id: wasmBucket.id(),
				name: wasmBucket.name(),
				storageClass: wasmBucket.storageClass(),
				bucketType: wasmBucket.bucketType(),
				files: [],
				snapshots: [],
				keys: [],
				locked: false,
				isSnapshotValid: false
			};

			setBuckets(prev => [...prev, bucket].sort((a, b) => a.name.localeCompare(b.name)));
			return bucket.id
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
	const restore = async (bucket: Bucket, snapshot: WasmSnapshot) => await tombMutex(bucket.mount!, async mount => await mount.restore(snapshot));

	/** Generates public link to share file. */
	const shareFile = async (bucket: Bucket, path: string[]) => await tombMutex(bucket.mount!, async mount => await mount.shareFile(path));

	/** Approves access key for bucket */
	const approveBucketAccess = async (bucket: Bucket, bucket_key_id: string) => {
		await tombMutex(bucket.mount!, async mount => {
			await mount.shareWith(bucket_key_id);
		});
		await getBucketsKeys();
	};

	/** Returns list of snapshots for selected bucket */
	const getBucketShapshots = async (id: string) => await tombMutex(tomb, async tomb => await tomb!.listBucketSnapshots(id));

	/** Approves a new deviceKey */
	const approveDeviceApiKey = async (pem: string) => await tombMutex(tomb, async tomb => await tomb!.approveDeviceApiKey(pem));

	/** Deletes access key for bucket */
	const removeBucketAccess = async (id: string) => {
		/** TODO:  connect removeBucketAccess method when in will be implemented.  */
		await getBucketsKeys();
	};

	const purgeSnapshot = async (id: string) => {
		// await tomb.purgeSnapshot(id);
	};

	/** Renames bucket */

	const moveTo = async (bucket: Bucket, from: string[], to: string[], name: string) => {
		return await await tombMutex(bucket.mount!, async mount => {
			const extstingFiles = (await mount.ls(to)).map(file => file.name);
			const browserObjectName = handleNameDuplication(name, extstingFiles);
			await mount.mv(from, [...to, browserObjectName]);
			const isSnapshotValid = await mount.hasSnapshot();
			await updateBucketsState('isSnapshotValid', isSnapshotValid, bucket.id);
		});
	};

	/** Internal function which looking for selected bucket and updates it, or bucket in buckets list if no bucket selected. */
	const updateBucketsState = (key: 'keys' | 'files' | 'snapshots' | 'isSnapshotValid', elements: BrowserObject[] | BucketSnapshot[] | boolean, id: string,) => {
		/** If we are on buckets list screen there is no selected bucket in state. */
		if (selectedBucket?.id === id) {
			setSelectedBucket(bucket => bucket ? { ...bucket, [key]: elements } : bucket);
		};

		setBuckets(buckets => buckets.map(bucket => {
			if (bucket.id === id) {
				return { ...bucket, [key]: elements };
			}

			return bucket;
		}));
	};

	const renameBucket = async (bucket: Bucket, newName: string) => {
		return await tombMutex(bucket.mount!, async mount => {
			await mount.rename(newName);
			bucket.name = newName;
			setBuckets(prev => prev.map(element => element.id === bucket.id ? { ...element, name: newName } : element));
		});
	};

	/** Creates directory inside selected bucket */
	const createDirectory = async (bucket: Bucket, path: string[], name: string) => {
		return await tombMutex(bucket.mount!, async mount => {
			await mount.mkdir([...path, name]);
			if (path.join('') !== folderLocation.join('')) { return; }
			const files = await mount.ls(path) || [];
			await updateBucketsState('files', files.sort(sortByType), bucket.id);
			const isSnapshotValid = await mount.hasSnapshot();
			await updateBucketsState('isSnapshotValid', isSnapshotValid, bucket.id);
		});
	};

	const getStorageUsageState = async () => {
		const usage = await storageUsageClient.getStorageUsage();
		const limit = await storageUsageClient.getStorageLimits();
		setStorageUsage({
			usage: usage.size,
			softLimit: limit.soft_hot_storage_limit,
			hardLimit: limit.hard_hot_storage_limit
		});
	};

	/** Uploads file to selected bucket/directory, updates buckets state */
	const uploadFile = async (bucket: Bucket, uploadPath: string[], name: string, file: ArrayBuffer, folder?: BrowserObject) => {
		return await tombMutex(bucket.mount!, async mount => {
			const extstingFiles = (await mount.ls(uploadPath)).map(file => file.name);
			let fileName = handleNameDuplication(name, extstingFiles);
			await mount.write([...uploadPath, fileName], file);
			if (folder) {
				const files = await mount.ls(uploadPath);
				folder.files = files.sort(sortByType);
				setSelectedBucket(prev => ({ ...prev! }));

				return;
			}
			if (uploadPath.join('') !== folderLocation.join('')) { return; }
			const files = await mount.ls(uploadPath) || [];
			await updateBucketsState('files', files.sort(sortByType), bucket.id);
			const isSnapshotValid = await mount.hasSnapshot();
			await updateBucketsState('isSnapshotValid', isSnapshotValid, bucket.id);
			await getStorageUsageState();
		});
	};

	/** Creates bucket snapshot */
	const takeColdSnapshot = async (bucket: Bucket) => {
		return await tombMutex(tomb, async tomb => {
			await bucket.mount!.snapshot();
			const snapshots = await tomb!.listBucketSnapshots(bucket.id);
			await updateBucketsState('snapshots', snapshots, bucket.id);
			const isSnapshotValid = await bucket.mount!.hasSnapshot();
			await updateBucketsState('isSnapshotValid', isSnapshotValid, bucket.id);
		});
	};

	const deleteBucket = async (id: string) => {
		await tomb?.deleteBucket(id);
		await getBuckets();
		await getStorageUsageState();
		if (selectedBucket?.id === id) {
			navigate('/')
		}
	};

	const deleteFile = async (bucket: Bucket, path: string[], name: string) => {
		return await tombMutex(bucket.mount!, async mount => {
			await mount.rm([...path, name]);
			const isSnapshotValid = await mount.hasSnapshot();
			await updateBucketsState('isSnapshotValid', isSnapshotValid, bucket.id);
		});
	};

	// Initialize the tomb client
	useEffect(() => {
		if (!user || !escrowedKeyMaterial || !keystoreInitialized) { return; }

		(async () => {
			try {
				const apiKey = await getApiKey();
				const TombWasm = (await import('tomb-wasm-experimental')).TombWasm;
				const tomb = new TombWasm(
					apiKey.privatePem,
					user.id,
					window.location.protocol + '//' + window.location.host,
				);
				setTomb(await tomb);
			} catch (error: any) {
				dispatch(setError(new BannerError(error.message)));
			}
		})();
	}, [user, keystoreInitialized, isLoading, escrowedKeyMaterial]);

	useEffect(() => {
		if (!areTermsAccepted) return;

		if (!keystoreInitialized && !isLoading && !isLoggingOut) {
			openEscrowModal(!!escrowedKeyMaterial);
		};
	}, [isLoading, keystoreInitialized, areTermsAccepted, escrowedKeyMaterial, isLoggingOut]);

	useEffect(() => {
		const userClient = new UserClient();
		const termsClient = new TermsAndColditionsClient();
		(async () => {
			try {
				const termsAndConditions = await termsClient.getTermsAndCondition();
				const user = await userClient.getCurrentUser();

				if (!user) return;

				if (!user.acceptedTosAt) {
					openModal(
						<TermaAndConditions
							acceptTerms={setAreTermsAccepted}
							userData={user}
						/>, null, true, '', false);

					return;
				};

				if (user.acceptedTosAt <= +termsAndConditions.tos_date) {
					openModal(
						<TermsAndConditionsModal
							setAreTermsAccepted={setAreTermsAccepted}
							terms={termsAndConditions.tos_content}
							userData={user} />
						, null, true, '', false);

					return;
				};

				setAreTermsAccepted(true);
			} catch (error: any) {
				console.log(error);
			}

		})()
	}, [user])

	useEffect(() => {
		if (tomb) {
			(async () => {
				try {
					await getBuckets();
					await getStorageUsageState();
				} catch (error: any) {
					dispatch(setError(new BannerError(error.message)));
				}
			})();
		};
	}, [tomb]);

	return (
		<TombContext.Provider
			value={{
				tomb, buckets, storageUsage, trash, areBucketsLoading, selectedBucket,
				getBuckets, getBucketsFiles, getBucketsKeys, selectBucket, getSelectedBucketFiles,
				takeColdSnapshot, getBucketShapshots, createBucketAndMount, deleteBucket, remountBucket,
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
