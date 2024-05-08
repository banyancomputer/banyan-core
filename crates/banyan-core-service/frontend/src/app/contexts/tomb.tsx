import React, { ReactNode, createContext, useContext, useEffect, useState } from 'react';
import { TombWasm, WasmBucket, WasmBucketAccess, WasmUserKeyAccess } from 'tomb-wasm-experimental';
import { unwrapResult } from '@reduxjs/toolkit';
import { useNavigate } from 'react-router-dom';

import {
    BrowserObject, Bucket,
    BucketSnapshot,
} from '@/app/types/bucket';
import {
    UserAccessKey,
} from '@/app/types/userAccessKeys';
import { useFolderLocation } from '@/app/hooks/useFolderLocation';
import { destroyIsUserNew, getIsUserNew, sortByName, sortByType } from '@app/utils';
import { handleNameDuplication } from '@utils/names';
import { StorageUsageClient } from '@/api/storageUsage';
import { useAppDispatch, useAppSelector } from '../store';
import { BannerError, setError } from '@store/errors/slice';
import { getApiKey } from '@store/keystore/actions';
import { ToastNotifications } from '@utils/toastNotifications';
import { SnapshotsClient } from '@/api/snapshots';
import { StorageLimits, StorageUsage } from '@/entities/storage';

interface TombInterface {
    tomb: TombWasm | null;
    buckets: Bucket[];
    userAccessKeys: UserAccessKey[];
    storageUsage: StorageUsage;
    storageLimits: StorageLimits;
    trash: Bucket | null;
    areBucketsLoading: boolean;
    areAccessKeysLoading: boolean;
    selectedBucket: Bucket | null;
    getBuckets: () => Promise<Bucket[]>;
    getBucketsFiles: () => Promise<void>;
    getUserAccessKeys: () => Promise<void>;
    remountBucket: (bucket: Bucket) => Promise<void>;
    selectBucket: (bucket: Bucket | null) => void;
    getSelectedBucketFiles: (path: string[]) => void;
    getExpandedFolderFiles: (path: string[], folder: BrowserObject, bucket: Bucket) => Promise<void>;
    takeColdSnapshot: (bucket: Bucket) => Promise<void>;
    getBucketSnapshots: (id: string) => Promise<BucketSnapshot[]>;
    createDriveAndMount: (name: string, storageClass: string, bucketType: string) => Promise<string>;
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
    createAccessKey: (name: string, pem: string) => Promise<void>;
    approveBucketAccess: (bucket: Bucket, userKeyId: string) => Promise<void>;
    removeBucketAccess: (bucket: Bucket, userKeyId: string) => Promise<void>;
    restore: (bucket: Bucket, snapshotId: string) => Promise<void>;
};
const storageUsageClient = new StorageUsageClient();
const snapshotsClient = new SnapshotsClient();

const TombContext = createContext<TombInterface>({} as TombInterface);

export const TombProvider = ({ children }: { children: ReactNode }) => {
    const dispatch = useAppDispatch();
    const { user } = useAppSelector(state => state.session);
    const { keystoreInitialized, escrowedKeyMaterial } = useAppSelector(state => state.keystore);
    const navigate = useNavigate();
    const [tomb, setTomb] = useState<TombWasm | null>(null);
    const [buckets, setBuckets] = useState<Bucket[]>([]);
    const [userAccessKeys, setUserAccessKeys] = useState<UserAccessKey[]>([]);
    const [trash, setTrash] = useState<Bucket | null>(null);
    const [selectedBucket, setSelectedBucket] = useState<Bucket | null>(null);
    const [storageUsage, setStorageUsage] = useState<StorageUsage>(new StorageUsage());
    const [storageLimits, setStorageLimits] = useState<StorageLimits>(new StorageLimits());
    const [areBucketsLoading, setAreBucketsLoading] = useState<boolean>(true);
    const [areAccessKeysLoading, setAreAccessKeysLoading] = useState<boolean>(true);
    const folderLocation = useFolderLocation();
    const { driveAlreadyExists, folderAlreadyExists } = useAppSelector(state => state.locales.messages.contexts.tomb);

    /** Returns list of buckets. */
    const getBuckets = async () => {
        setAreBucketsLoading(true);
        const key = unwrapResult(await dispatch(getApiKey()));
        const wasm_buckets: WasmBucket[] = await tomb!.listBuckets();
        if (getIsUserNew()) {
            createDriveAndMount("My Drive", 'hot', 'interactive');
            destroyIsUserNew();
            return [];
        }
        const buckets: Bucket[] = [];
        for (let bucket of wasm_buckets) {
            const snapshots = await snapshotsClient.getSnapshots(bucket.id());
            let mount = await tomb!.mount(bucket.id(), key.privatePem);
            let locked = await mount.locked();
            let isSnapshotValid = await mount.hasSnapshot();
            buckets.push({
                mount: mount || null,
                id: bucket.id(),
                name: bucket.name(),
                storageClass: bucket.storageClass(),
                bucketType: bucket.bucketType(),
                files: [],
                snapshots,
                locked: locked || false,
                isSnapshotValid: isSnapshotValid || false
            });
        };
        setBuckets(buckets);

        setAreBucketsLoading(false);
        return buckets;
    };

    /** Pushes files and snapshots inside of buckets list. */
    const getBucketsFiles = async () => {
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
    };

    const remountBucket = async (bucket: Bucket) => {
        const key = unwrapResult(await dispatch(getApiKey()));
        const mount = await tomb!.mount(bucket.id, key.privatePem);
        const locked = await mount.locked();
        const isSnapshotValid = await mount.hasSnapshot();
        setBuckets(prev => prev.map(element => element.id === bucket.id ? { ...element, mount, locked, isSnapshotValid } : element));
    };

    /** Sets selected bucket into state. */
    const selectBucket = async (bucket: Bucket | null) => {
        setSelectedBucket(bucket);
    };

    /** Returns list of access keys. */
    const getUserAccessKeys = async () => {
        setAreAccessKeysLoading(true);
        const rawAccessKeys: WasmUserKeyAccess[] = await tomb!.userKeyAccess();
        const buckets = await getBuckets();
        setUserAccessKeys(rawAccessKeys.map(accessKey => {
            const key = accessKey.key;
            const keyBuckets = accessKey.bucketIds.map(bucketId => buckets.find(bucket => bucket.id === bucketId)!);

            return {
                id: key.id(),
                name: key.name(),
                userId: key.userId(),
                pem: key.pem(),
                fingerprint: key.fingerprint(),
                apiAccess: key.apiAccess(),
                createdAt: key.createdAt(),
                buckets: keyBuckets,
            }
        }));
        setAreAccessKeysLoading(false);
    }

    /** Returns selected bucket state according to current folder location. */
    const getSelectedBucketFiles = async (path: string[]) => {
        setAreBucketsLoading(true);
        const files = await selectedBucket?.mount?.ls(path);
        await setSelectedBucket(bucket => ({ ...bucket!, files: files ? files.sort(sortByName).sort(sortByType) : [] }));
        setAreBucketsLoading(false);
    };

    /** Returns selected folder files. */
    const getExpandedFolderFiles = async (path: string[], folder: BrowserObject, bucket: Bucket) => {
        const files = await selectedBucket?.mount?.ls(path);
        folder.files = files ? files.sort(sortByName).sort(sortByType) : [];
        setSelectedBucket(prev => ({ ...prev! }));
    };

    /** Creates new bucket with recieved parameters of type and storag class. */
    const createDriveAndMount = async (name: string, storageClass: string, bucketType: string): Promise<string> => {
        const existingBuckets = buckets.map(bucket => bucket.name);

        if (existingBuckets.includes(name)) {
            ToastNotifications.error(driveAlreadyExists);
            throw new Error(driveAlreadyExists);
        }

        const key = unwrapResult(await dispatch(getApiKey()));
        const { bucket: wasmBucket, mount: wasmMount } = await tomb!.createBucketAndMount(name, storageClass, bucketType, key.privatePem, key.publicPem);
        const bucket = {
            mount: wasmMount,
            id: wasmBucket.id(),
            name: wasmBucket.name(),
            storageClass: wasmBucket.storageClass(),
            bucketType: wasmBucket.bucketType(),
            files: [],
            snapshots: [],
            access: [],
            locked: false,
            isSnapshotValid: false
        };
        setBuckets(prev => [...prev, bucket].sort((a, b) => a.name.localeCompare(b.name)));
        return bucket.id;
    };

    /** Returns file as ArrayBuffer */
    const getFile = async (bucket: Bucket, path: string[], name: string) => await bucket.mount!.readBytes([...path, name]);

    /** Downloads file. */
    const download = async (bucket: Bucket, path: string[], name: string) => {
        const link = document.createElement('a');
        const arrayBuffer: Uint8Array = await getFile(bucket, path, name);
        const blob = new Blob([arrayBuffer]);
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

    /** Restores bucket from selected snapshot. */
    const restore = async (bucket: Bucket, snapshotId: string) => await snapshotsClient.restoreFromSnapshot(bucket.id, snapshotId);
    /** Approves access key for bucket */
    const approveBucketAccess = async (bucket: Bucket, bucketKeyId: string) => {
    };

    /** Generates public link to share file. */
    const shareFile = async (bucket: Bucket, path: string[]) => await bucket.mount!.shareFile(path);
    /** Returns list of snapshots for selected bucket. */
    const getBucketSnapshots = async (id: string) => await snapshotsClient.getSnapshots(id);

    /** Deletes access key for bucket */
    const removeBucketAccess = async (bucket: Bucket, fingerprint: string) => {
        /** TODO:  connect removeBucketAccess method when in will be implemented.  */
    };

    /** Creates a new API authenticated UserKey */
    const createAccessKey = async (name: string, pem: string) => {
        await tomb!.createUserKey(name, pem);
    };

    /** Moves file into different location. */
    const moveTo = async (bucket: Bucket, from: string[], to: string[], name: string) => {
        const mount = bucket.mount!;
        const extstingFiles = (await mount.ls(to)).map(file => file.name);
        const browserObjectName = handleNameDuplication(name, extstingFiles);
        await mount.mv(from, [...to, browserObjectName]);
        const isSnapshotValid = await mount.hasSnapshot();
        await updateBucketsState('isSnapshotValid', isSnapshotValid, bucket.id);
    };

    const purgeSnapshot = async (id: string) => {
        // await tomb.purgeSnapshot(id);
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

    /** Changes name of bucket. */
    const renameBucket = async (bucket: Bucket, newName: string) => {
        await bucket.mount!.rename(newName);
        bucket.name = newName;
        setBuckets(prev => prev.map(element => element.id === bucket.id ? { ...element, name: newName } : element));
    };

    /** Creates directory inside selected bucket */
    const createDirectory = async (bucket: Bucket, path: string[], name: string) => {
        const mount = bucket.mount!;
        const extstingFolders = (await mount.ls(path)).map(file => file.name);

        if (extstingFolders.includes(name)) {
            ToastNotifications.error(folderAlreadyExists);

            throw new Error(folderAlreadyExists);
        };

        await mount.mkdir([...path, name]);
        if (path.join('') !== folderLocation.join('')) { return; }
        const files = await mount.ls(path) || [];
        await updateBucketsState('files', files.sort(sortByName).sort(sortByType), bucket.id);
        const isSnapshotValid = await mount.hasSnapshot();
        await updateBucketsState('isSnapshotValid', isSnapshotValid, bucket.id);
    };

    /** Gets storage usage info and sets it into state. */
    const updateStorageUsageState = async () => {
        try {
            const usage = await storageUsageClient.getStorageUsage();
            setStorageUsage(usage);
        } catch (error: any) { };
    };

    const updateStorageLimitsState = async () => {
        try {
            const limits = await storageUsageClient.getStorageLimits();
            setStorageLimits(limits);
        } catch (error: any) { };
    };

    /** Uploads file to selected bucket/directory, updates buckets state */
    const uploadFile = async (bucket: Bucket, uploadPath: string[], name: string, file: ArrayBuffer, folder?: BrowserObject) => {
        const mount = bucket.mount!;
        const extstingFiles = (await mount.ls(uploadPath)).map(file => file.name);
        const fileName = handleNameDuplication(name, extstingFiles);
        await mount.write([...uploadPath, fileName], file);
        if (folder) {
            const files = await mount.ls(uploadPath);
            folder.files = files.sort(sortByName).sort(sortByType);
            setSelectedBucket(prev => ({ ...prev! }));

            return;
        }
        if (uploadPath.join('') !== folderLocation.join('')) { return; }
        const files = await mount.ls(uploadPath) || [];
        await updateBucketsState('files', files.sort(sortByName).sort(sortByType), bucket.id);
        const isSnapshotValid = await mount.hasSnapshot();
        await updateBucketsState('isSnapshotValid', isSnapshotValid, bucket.id);
        await updateStorageUsageState();
    };

    /** Creates bucket snapshot */
    const takeColdSnapshot = async (bucket: Bucket) => {
        await bucket.mount!.snapshot();
        const snapshots = await tomb!.listBucketSnapshots(bucket.id);
        await updateBucketsState('snapshots', snapshots, bucket.id);
        const isSnapshotValid = await bucket.mount!.hasSnapshot();
        await updateBucketsState('isSnapshotValid', isSnapshotValid, bucket.id);
        await updateStorageUsageState();
    };

    const deleteBucket = async (id: string) => {
        await tomb?.deleteBucket(id);
        await getBuckets();
        await updateStorageUsageState();
        selectedBucket?.id === id && navigate('/');
        await updateStorageUsageState();
    };

    const deleteFile = async (bucket: Bucket, path: string[], name: string) => {
        const mount = bucket.mount!;
        await mount.rm([...path, name]);
        const isSnapshotValid = await mount.hasSnapshot();
        await updateBucketsState('isSnapshotValid', isSnapshotValid, bucket.id);
        await updateStorageUsageState();
    };

    // Initialize the tomb client
    useEffect(() => {
        if (!user.id || !keystoreInitialized || !escrowedKeyMaterial) { return; }

        (async () => {
            try {
                const apiKey = unwrapResult(await dispatch(getApiKey()));
                const tomb = await new TombWasm(
                    apiKey.privatePem,
                    user.id,
                    window.location.protocol + '//' + window.location.host,
                );
                setTomb(tomb);
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
                } catch (error: any) {
                    dispatch(setError(new BannerError(error.message)));
                    setAreBucketsLoading(false);
                }
            })();
        };
    }, [tomb]);

    return (
        <TombContext.Provider
            value={{
                tomb, buckets, userAccessKeys, areAccessKeysLoading, storageUsage, storageLimits, trash, areBucketsLoading, selectedBucket,
                getBuckets, getBucketsFiles, getUserAccessKeys, selectBucket, getSelectedBucketFiles,
                takeColdSnapshot, getBucketSnapshots, createDriveAndMount, deleteBucket, remountBucket,
                getFile, renameBucket, createDirectory, uploadFile, purgeSnapshot,
                removeBucketAccess, approveBucketAccess, createAccessKey, shareFile, download, moveTo,
                restore, deleteFile, makeCopy, getExpandedFolderFiles,
            }}
        >
            {children}
        </TombContext.Provider>
    );
};

export const useTomb = () => useContext(TombContext);
