import React, { ReactNode, createContext, useContext, useEffect, useState } from 'react';
import { TombWasm, WasmBucket, WasmMount, WasmSnapshot } from 'tomb-wasm-experimental';
import { Mutex } from 'async-mutex';
import { useSession } from 'next-auth/react';

import { useKeystore } from './keystore';
import {
    Bucket, BucketKey,
    BucketSnapshot, FileMetadata, MockBucket,
} from '@/lib/interfaces/bucket';
import { useFolderLocation } from '@/hooks/useFolderLocation';

interface TombInterface {
    tomb: TombWasm | null;
    buckets: Array<Bucket>;
    usedStorage: number;
    usageLimit: number;
    trash: Bucket;
    isTrashLoading: boolean;
    areBucketsLoading: boolean;
    selectedBucket: Bucket | null;
    selectBucket: (bucket: Bucket | null) => void;
    getSelectedBucketFiles: (path: string[]) => void;
    download: (bucket: Bucket, path: string[], name: string) => Promise<void>;
    getFile: (bucket: Bucket, path: string[], name: string) => Promise<ArrayBuffer>;
    shareWith: (bucket: Bucket, key: string) => Promise<void>;
    copyToClipboard: (bucket: Bucket, path: string[], name: string) => void;
    takeColdSnapshot: (bucket: Bucket) => Promise<void>;
    getBuckets: () => Promise<void>;
    moveTo: (bucket: Bucket, from: string[], to: string[]) => Promise<void>;
    createBucket: (name: string, storageClass: string, bucketType: string) => Promise<void>;
    createDirectory: (bucket: Bucket, path: string[], name: string) => Promise<void>;
    uploadFile: (id: string, path: string[], name: string, file: any, folderLocation: string[]) => Promise<void>;
    getTrashBucket: () => Promise<void>;
    getBucketShapshots: (id: string) => Promise<BucketSnapshot[]>;
    getBucketKeys: (id: string) => Promise<BucketKey[]>;
    purgeSnapshot: (id: string) => void;
    deleteBucket: (id: string) => void;
    deleteFile: (bucket: Bucket, path: string[], name: string) => void;
    approveBucketAccess: (id: string) => Promise<void>;
    removeBucketAccess: (id: string) => Promise<void>;
    restore: (bucket: Bucket, snapshot: WasmSnapshot) => Promise<void>;
};

type TombBucket = Bucket & { mount: WasmMount };
const mutex = new Mutex();

const TombContext = createContext<TombInterface>({} as TombInterface);

export const TombProvider = ({ children }: { children: ReactNode }) => {
    const { data: session } = useSession();
    const { keystoreInitialized, getEncryptionKey, getApiKey, escrowedDevice } = useKeystore();
    const [tomb, setTomb] = useState<TombWasm | null>(null);
    const [buckets, setBuckets] = useState<Array<TombBucket>>([]);
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
    }

    /** Returns list of buckets. */
    const getBuckets = async () => {
        setAreBucketsLoading(true);
        tombMutex(tomb, async tomb => {
            const wasm_buckets: WasmBucket[] = await tomb!.listBuckets();
            const buckets = [];
            let key = await getEncryptionKey();
            for (let bucket of wasm_buckets) {
                const mount = await tomb!.mount(bucket.id(), key);
                const files = await mount.ls([]);
                const keys = await tomb!.listBucketKeys(bucket.id());
                buckets.push({
                    mount,
                    id: bucket.id(),
                    name: bucket.name(),
                    storageClass: bucket.storageClass(),
                    bucketType: bucket.bucketType(),
                    files: files || [],
                    keys,
                });
            }

            setBuckets(buckets);
            setAreBucketsLoading(false);
        });
    };

    /** Creates new bucket with recieved parameters of type and storag class. */
    const createBucket = async (name: string, storageClass: string, bucketType: string) => {
        await tombMutex(tomb, async tomb => {
            let key = await getEncryptionKey();
            let wasmBucket = await tomb!.createBucket(name, storageClass, bucketType, key.publicKey);
            let mount = await tomb!.mount(wasmBucket.id(), key);
            const files = await mount.ls([]);
            const keys = await tomb!.listBucketKeys(wasmBucket.id())
            let bucket = {
                mount,
                id: wasmBucket.id(),
                name: wasmBucket.name(),
                storageClass: wasmBucket.storageClass(),
                bucketType: wasmBucket.bucketType(),
                files: files || [],
                keys,
            }

            setBuckets(prev => [...prev, bucket]);
        })
    };

    /** Returns file as ArrayBuffer */
    const getFile = async (bucket: Bucket, path: string[], name: string) => {
        return await tombMutex(bucket.mount, async mount => await mount!.readBytes([...path, name]));
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

    /** Copies file to clipboard. */
    const copyToClipboard = async (bucket: Bucket, path: string[], name: string) => {
        const arrayBuffer: ArrayBuffer = await getFile(bucket, path, name);
        const blob = new Blob([arrayBuffer], { type: 'application/octet-stream' });
        // navigator.clipboard.write([new ClipboardItem({ '': blob })])
    };

    /** Retuns array buffer of selected file. */
    const restore = async (bucket: Bucket, snapshot: WasmSnapshot) => await tombMutex(bucket.mount, async mount => await mount.restore(snapshot));

    /** Shares bucket with selected key. */
    const shareWith = async (bucket: Bucket, key: string) => await tombMutex(bucket.mount, async mount => await mount.shareWith(key));

    const getBucketKeys = async (id: string) => await tombMutex(tomb, async tomb => await tomb!.listBucketKeys(id));

    /** Returns list of snapshots for selected bucket */
    const getBucketShapshots = async (id: string) => await tombMutex(tomb, async tomb => await tomb!.listBucketSnapshots(id));

    /** Approves access key for bucket */
    const approveBucketAccess = async (id: string) => {
        /** TODO:  connect approveBucketAccess method when in will be implemented.  */
        // await tomb.approveBucketAccess(id);
    };

    /** Deletes access key for bucket */
    const removeBucketAccess = async (id: string) => {
        /** TODO:  connect removeBucketAccess method when in will be implemented.  */
        // return await tomb.approveBucketAccess(id);
    };

    const purgeSnapshot = async (id: string) => {
        // await tomb.purgeSnapshot(id);
    };
    /** Sets selected bucket into state */
    const selectBucket = async (bucket: Bucket | null) => {
        setSelectedBucket(bucket);
    };

    /** Returns selected bucket state according to current folder location. */
    const getSelectedBucketFiles = async (path: string[]) => {
        if (!selectedBucket) return;

        tombMutex(selectedBucket.mount, async mount => {
            const files = await mount.ls(path);
            await setSelectedBucket(bucket => bucket ? ({ ...bucket, files }) : bucket);
        });
    };

    /** Renames bucket */
    const moveTo = async (bucket: Bucket, from: string[], to: string[]) => {
        await tombMutex(bucket.mount, async mount => {
            await mount.mv(from, to);
        });
    };
    /** Creates directory inside selected bucket */
    const createDirectory = async (bucket: Bucket, path: string[], name: string) => {
        await tombMutex(bucket.mount, async mount => {
            const id = bucket.id;
            await mount.mkdir([...path, name]);
            if (path.join('') !== folderLocation.join('')) return;
            const files = await mount.ls(path) || [];
            if (selectedBucket) {
                setSelectedBucket(bucket => bucket ? ({ ...bucket, files }) : bucket);
                return;
            };

            setBuckets(buckets => buckets.map(bucket => {
                if (bucket.id === id) {
                    return ({ ...bucket, files })
                }
                return bucket;
            }))
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
    const uploadFile = async (id: string, path: string[], name: string, file: ArrayBuffer) => {
        try {
            const bucket = buckets.find(bucket => bucket.id == id);
            tombMutex(bucket!.mount, async mount => {
                await mount.write([...path, name], file);
                if (path.join('') !== folderLocation.join('')) return;
                const files = await mount.ls(path) || [];

                if (selectedBucket) {
                    setSelectedBucket(bucket => bucket ? ({ ...bucket, files }) : bucket);
                    return;
                };

                setBuckets(buckets => buckets.map(bucket => {
                    if (bucket.id === id) {
                        return ({ ...bucket, files: files })
                    }
                    return bucket;
                }))
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
    }

    const deleteFile = async (bucket: Bucket, path: string[], name: string) => {
        await tombMutex(bucket.mount, async mount => {
            await mount.rm([...path, name]);
            await getSelectedBucketFiles(path);
        });
    };

    useEffect(() => {
        if (!selectedBucket) return
        selectBucket(buckets.find(bucket => bucket.id === selectedBucket.id)!)
    }, [buckets, selectedBucket?.id]);

    // Initialize the tomb client
    useEffect(() => {
        if (!keystoreInitialized || !session?.accountId || !escrowedDevice) { return; }
        (async () => {
            try {
                const apiKey = await getApiKey();
                const TombWasm = (await import('tomb-wasm-experimental')).TombWasm;
                const tomb = new TombWasm(
                    apiKey,
                    session.accountId,
                    process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001',
                    process.env.NEXT_PUBLIC_DATA_URL || 'http://localhost:3002',
                );
                setTomb(tomb);
            } catch (err) {
                console.error(err);
            }
        })();
    }, [keystoreInitialized, session?.accountId, escrowedDevice]);

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
                getBuckets, getBucketShapshots, createBucket, deleteBucket, selectBucket, getFile,
                getTrashBucket, takeColdSnapshot, createDirectory,
                uploadFile, getBucketKeys, purgeSnapshot, getSelectedBucketFiles,
                removeBucketAccess, approveBucketAccess,
                shareWith, download, moveTo, restore, deleteFile, copyToClipboard
            }}
        >
            {children}
        </TombContext.Provider>
    );
};

export const useTomb = () => useContext(TombContext);
