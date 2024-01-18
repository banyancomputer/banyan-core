import { ReactNode, createContext, useContext, useEffect, useState } from 'react';
import { TombWasm, WasmSnapshot } from 'tomb-wasm-experimental';
import { Mutex } from 'async-mutex';
import { useNavigate } from 'react-router-dom';
import { wrap } from 'comlink';

import { TermsAndConditionsModal } from '@components/common/Modal/TermsAndConditionsModal';
import { TermaAndConditions } from '@components/common/TermsAndConditions';

import { useModal } from '@/app/contexts/modals';
import { useKeystore } from './keystore';
import { BrowserObject, Bucket,	BucketSnapshot } from '@/app/types/bucket';
import { useFolderLocation } from '@/app/hooks/useFolderLocation';
import { useSession } from './session';
import { destroyIsUserNew, getIsUserNew } from '@app/utils';
import { TermsAndColditionsClient } from '@/api/termsAndConditions';
import { UserClient } from '@/api/user';

interface TombInterface {
	tomb: TombWasm | null;
	buckets: Bucket[];
	storageUsage: { current: number, limit: number };
	trash: Bucket | null;
	areBucketsLoading: boolean;
	selectedBucket: Bucket | null;
	error: string;
	getBuckets: () => Promise<void>;
	getBucketsFiles: () => Promise<void>;
	getBucketsKeys: () => Promise<void>;
	remountBucket: (bucket: Bucket) => Promise<void>;
	selectBucket: (bucket: Bucket | null) => void;
	getSelectedBucketFiles: (path: string[]) => void;
	getSelectedBucketFolders: (bucketId: string, path: string[]) => Promise<BrowserObject[]>;
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

const mutex = new Mutex();

const worker = new Worker(new URL('../../workers/tomb.worker.ts', import.meta.url));
const tombWorker = wrap<import('../../workers/tomb.worker.ts').TombWorker>(worker);

const TombContext = createContext<TombInterface>({} as TombInterface);

export const TombProvider = ({ children }: { children: ReactNode }) => {
	const { userData } = useSession();
	const navigate = useNavigate();
	const { openEscrowModal, openModal } = useModal();
	const { isLoading, keystoreInitialized, getEncryptionKey, getApiKey, escrowedKeyMaterial } = useKeystore();
	const [tomb, setTomb] = useState<TombWasm | null>(null);
	const [buckets, setBuckets] = useState<Bucket[]>([]);
	const [trash, setTrash] = useState<Bucket | null>(null);
	const [areTermsAccepted, setAreTermsAccepted] = useState(false);
	const [selectedBucket, setSelectedBucket] = useState<Bucket | null>(null);
	const [storageUsage, setStorageUsage] = useState<{ current: number, limit: number }>({ current: 0, limit: 0 });
	const [areBucketsLoading, setAreBucketsLoading] = useState<boolean>(false);
	const folderLocation = useFolderLocation();
	const [error, setError] = useState<string>('');

	/** Prevents rust recursion error. */
	const tombMutex = async <T,>(tomb: T, callback: (tomb: T) => Promise<any>) => {
		const release = await mutex.acquire();
		try {
			return await callback(tomb);
		} catch (error) {
			console.error('tombMutex', error);
			setAreBucketsLoading(false);
		} finally {
			release();
		}
	};

	/** Returns list of buckets. */
	const getBuckets = async () => {
		await tombWorker.getBuckets();
	};

	/** Pushes files and snapshots inside of buckets list. */
	const getBucketsFiles = async () => {
		await tombWorker.getBucketsFiles();
	};

	const remountBucket = async (bucket: Bucket) => {
		tombMutex(tomb, async tomb => {
			const key = await getEncryptionKey();
			const mount = await tomb!.mount(bucket.id, key.privatePem);
			const locked = await mount.locked();
			const isSnapshotValid = await mount.hasSnapshot();
			setBuckets(prev => prev.map(element => element.id === bucket.id ? { ...element, mount, locked, isSnapshotValid } : element));
		});
	};

	/** Pushes keys inside of buckets list. */
	const getBucketsKeys = async () => {
		await tombWorker.getBucketsKeys();
	};

	/** Returns selected bucket state according to current folder location. */
	const getSelectedBucketFiles = async (path: string[]) => {
		await tombWorker.getSelectedBucketFiles(path);
	};

	/** Returns selected bucket folders state according to current folder location. */
	const getSelectedBucketFolders = async (bucketId: string, path: string[]) => {
		return await tombWorker.getSelectedBucketFolders(bucketId, path);
	};

	/** Returns selected folder files. */
	const getExpandedFolderFiles = async (path: string[], folder: BrowserObject, bucket: Bucket) => {
		// await tombMutex(selectedBucket!.mount!, async mount => {
		// 	const files = await mount.ls(path);
		// 	folder.files = files.sort(sortByType);
		// 	setSelectedBucket(prev => ({ ...prev! }));
		// });
	};

	/** Sets selected bucket into state */
	const selectBucket = async (bucket: Bucket | null) => {
		await tombWorker.selectBucket(bucket?.id || null);
	};

	/** Creates new bucket with recieved parameters of type and storag class. */
	const createBucketAndMount = async (name: string, storageClass: string, bucketType: string): Promise<string> => {
		return await tombWorker.createBucketAndMount(name, storageClass, bucketType);
	};

	/** Returns file as ArrayBuffer */
	const getFile = async (bucket: Bucket, path: string[], name: string) => {
		return await tombWorker.getFile(bucket.id, path, name);
	};

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

	/** Restores buckets state to snapshot vesnion. */
	const restore = async (bucket: Bucket, snapshot: WasmSnapshot) => {
		return await tombWorker.restore(bucket.id, snapshot);
	};

	/** Generates public link to share file. */
	const shareFile = async (bucket: Bucket, path: string[]) => {
		return await tombWorker.shareFile(bucket.id, path);
	};

	/** Approves access key for bucket */
	const approveBucketAccess = async (bucket: Bucket, bucket_key_id: string) => {
		await tombWorker.approveBucketAccess(bucket.id, bucket_key_id)
	};

	/** Returns list of snapshots for selected bucket */
	const getBucketShapshots = async (id: string) => {
		return await tombWorker.getBucketShapshots(id);
	};

	/** Approves a new deviceKey */
	const approveDeviceApiKey = async (pem: string) => {
		return await tombWorker.approveDeviceApiKey(pem);
	};

	/** Deletes access key for bucket */
	const removeBucketAccess = async (id: string) => {
		/** TODO:  connect removeBucketAccess method when in will be implemented.  */
		await getBucketsKeys();
	};

	const purgeSnapshot = async (id: string) => {
		// await tomb.purgeSnapshot(id);
	};

	/** Changes placement of file inside bucket tree scructure */
	const moveTo = async (bucket: Bucket, from: string[], to: string[], name: string) => {
		await tombWorker.moveTo(bucket.id, from, to, name);
	};

	const renameBucket = async (bucket: Bucket, newName: string) => {
		await tombWorker.renameBucket(bucket.id, newName);
	};

	/** Creates directory inside selected bucket */
	const createDirectory = async (bucket: Bucket, path: string[], name: string) => {
		await tombWorker.createDirectory(bucket.id, path, name, folderLocation);
	};

	/** Returns used storage amount in bytes */
	const getStorageUsageState = async () => {
		const { current, limit } = await tombWorker.getStorageUsageState();
		setStorageUsage({ current, limit });
	};

	/** Uploads file to selected bucket/directory, updates buckets state. */
	const uploadFile = async (bucket: Bucket, uploadPath: string[], name: string, file: ArrayBuffer, folder?: BrowserObject) => {
		await tombWorker.uploadFile(bucket.id, uploadPath, folderLocation, name, file, folder);
	};

	/** Creates bucket snapshot */
	const takeColdSnapshot = async (bucket: Bucket) => {
		await tombWorker.takeColdSnapshot(bucket.id);
	};

	const deleteBucket = async (id: string) => {
		await tombWorker.deleteBucket(id);
		if (selectedBucket?.id === id) {
			navigate('/');
		};
	};

	const deleteFile = async (bucket: Bucket, path: string[], name: string) => {
		await tombWorker.deleteFile(bucket.id, path, name);
	};


	// Initialize the tomb client
	useEffect(() => {
		if (!userData || !keystoreInitialized) { return; }

		(async () => {
			try {
				const apiKey = await getApiKey();
				const encryptionKey = await getEncryptionKey();
				/** Will create tomb instance in the worker stream. */
				await tombWorker.mountTomb(apiKey, userData.user.id, window.location.protocol + '//' + window.location.host, encryptionKey);
			} catch (error: any) {
				setError(error.message);
			}
		})();
	}, [userData, keystoreInitialized, isLoading, escrowedKeyMaterial]);

	useEffect(() => {
		if (!areTermsAccepted) return;

		if (!keystoreInitialized && !isLoading) {
			openEscrowModal(!!escrowedKeyMaterial);
		};
	}, [isLoading, keystoreInitialized, areTermsAccepted]);

	useEffect(() => {
		const userClient = new UserClient();
		const termsClient = new TermsAndColditionsClient();
		(async () => {
			try {
				const termsAndConditions = await termsClient.getTermsAndCondition();
				const userData = await userClient.getCurrentUser();

				if (!userData) return;

				if (!userData.accepted_tos_at) {
					openModal(
						<TermaAndConditions
							acceptTerms={setAreTermsAccepted}
							userData={userData}
						/>, null, true, '');

					return;
				};

				if (userData.accepted_tos_at <= +termsAndConditions.tos_date) {
					openModal(
						<TermsAndConditionsModal
							setAreTermsAccepted={setAreTermsAccepted}
							terms={termsAndConditions.tos_content}
							userData={userData} />
						, null, true);

					return;
				};

				setAreTermsAccepted(true);
			} catch (error: any) {
				console.log(error);
			}

		})()
	}, [userData])

	useEffect(() => {
		if (!tomb) { return };
		(async () => {
			try {
				await getBuckets();
				const isUserNew = getIsUserNew();

				if (isUserNew) {
					await createBucketAndMount("My Drive", 'hot', 'interactive');
					destroyIsUserNew();
				};
				await getStorageUsageState();
			} catch (error: any) {
				setError(error.message);
			}
		})();
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
				}
			};
		})()
	}, [])

	return (
		<TombContext.Provider
			value={{
				tomb, buckets, storageUsage, trash, areBucketsLoading, selectedBucket, error,
				getBuckets, getBucketsFiles, getBucketsKeys, selectBucket, getSelectedBucketFiles,
				takeColdSnapshot, getBucketShapshots, createBucketAndMount, deleteBucket, remountBucket,
				getFile, renameBucket, createDirectory, uploadFile, purgeSnapshot, getSelectedBucketFolders,
				removeBucketAccess, approveBucketAccess, approveDeviceApiKey, shareFile, download, moveTo,
				restore, deleteFile, makeCopy, getExpandedFolderFiles,
			}}
		>
			{children}
		</TombContext.Provider>
	);
};

export const useTomb = () => useContext(TombContext);
