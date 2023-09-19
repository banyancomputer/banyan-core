import React, { ReactNode, createContext, useContext, useEffect, useState } from 'react';
import { TombWasm, WasmBucket, WasmMount, WasmSnapshot } from 'tomb-wasm-experimental';
import { Mutex } from 'async-mutex';
import { useSession } from 'next-auth/react';

import { useKeystore } from './keystore';
import {
    Bucket, BucketKey,
    BucketSnapshot, MockBucket,
} from '@/lib/interfaces/bucket';

interface TombInterface {
    tomb: TombWasm | null;
    buckets: Array<Bucket>;
    usedStorage: number;
    usageLimit: number;
    trash: Bucket;
    isTrashLoading: boolean;
    areBucketsLoading: boolean;
    selectedBucket: Bucket | null;
    selectBucket: (bucket: Bucket) => void;
    getSelectedBucketFiles: (path: string[]) => void;
    download: (bucket: Bucket, path: string[], name: string) => Promise<void>;
    getFile: (bucket: Bucket, path: string[], name: string) => Promise<ArrayBuffer>;
    shareWith: (bucket: Bucket, key: string) => Promise<void>
    takeColdSnapshot: (bucket: Bucket) => Promise<void>;
    getBuckets: () => Promise<void>;
    moveTo: (bucket: Bucket, from: string[], to: string[]) => Promise<void>;
    createBucket: (name: string, storageClass: string, bucketType: string) => Promise<void>;
    createDirectory: (bucket: Bucket, path: string[]) => Promise<void>;
    uploadFile: (id: string, path: string[], name: string, file: any) => Promise<void>;
    renameFile: (id: string, path: string[], newPath: string[]) => Promise<void>;
    getTrashBucket: () => Promise<void>;
    getUsedStorage: () => Promise<number>;
    getUsageLimit: () => Promise<number>;
    getBucketShapshots: (id: string) => Promise<BucketSnapshot[]>;
    getBucketKeys: (id: string) => Promise<BucketKey[]>;
    purgeSnapshot: (id: string) => void;
    deleteBucket: (id: string) => void;
    deleteFile: (bucket: Bucket, path: string[]) => void;
    approveBucketAccess: (id: string) => Promise<void>;
    restore: (bucket: Bucket, snapshot: WasmSnapshot) => Promise<void>;
    removeBucketAccess: (id: string) => Promise<void>;
};

const mutex = new Mutex();

const TombContext = createContext<TombInterface>({} as TombInterface);

export const TombProvider = ({ children }: { children: ReactNode }) => {
    // The active user's session
    const { data: session } = useSession();
    // The active user's keystore
    const { keystoreInitialized, getEncryptionKey, getApiKey } = useKeystore();
    const [tomb, setTomb] = useState<TombWasm | null>(null);
    const [buckets, setBuckets] = useState<Array<Bucket & { mount: WasmMount }>>([]);
    const [trash, setTrash] = useState<Bucket>(new MockBucket());
    const [selectedBucket, setSelectedBucket] = useState<Bucket | null>(null);
    const [usedStorage, setUsedStorage] = useState<number>(0);
    const [usageLimit, setUsageLimit] = useState<number>(0);
    const [isTrashLoading, setIsTrashLoading] = useState<boolean>(true);
    const [areBucketsLoading, setAreBucketsLoading] = useState<boolean>(false);

    /** Prevents rust recursion error. */
    const tombMutex = async (calllack: (tomb: TombWasm) => Promise<any>) => {
        if (tomb) {
            const release = await mutex.acquire();
            try {
                return await calllack(tomb);
            } catch (err) {
                console.error('err', err);
            } finally {
                release();
            }
        }
    };

    const mountMutex = async (bucket: Bucket, callback: (mount: WasmMount) => Promise<any>) => {
        const release = await mutex.acquire();
        try {
            return await callback(bucket.mount);
        } catch (err) {
            console.error('err', err);
        } finally {
            release();
        }
    }

    /** Returns list of buckets. */
    const getBuckets = async () => {
        setAreBucketsLoading(true);
        tombMutex(async tomb => {
            const wasm_buckets: WasmBucket[] = await tomb!.listBuckets();
            const buckets = [];
            let key = await getEncryptionKey();
            for (let bucket of wasm_buckets) {
                const mount = await tomb.mount(bucket.id(), key);
                const files = await mount.ls([]);
                const keys = await tomb.listBucketKeys(bucket.id());
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
        await tombMutex(async tomb => {
            let key = await getEncryptionKey();
            let wasmBucket = await tomb!.createBucket(name, storageClass, bucketType, key.publicKey);
            let mount = await tomb!.mount(wasmBucket.id(), key);
            const files = await mount.ls([]);
            const keys = await tomb.listBucketKeys(wasmBucket.id())
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
        return await mountMutex(bucket, async mount => await mount!.readBytes([...path, name]));
    };

    /** Downloads file. */
    const download = async (bucket: Bucket, path: string[], name: string) => {
        const link = document.createElement('a');
        const arrayBuffer: ArrayBuffer = await getFile(bucket, path, name);
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
    const restore = async (bucket: Bucket, snapshot: WasmSnapshot) => await mountMutex(bucket, async mount => await mount.restore(snapshot));

    /** Shares bucket with selected key. */
    const shareWith = async (bucket: Bucket, key: string) => await mountMutex(bucket, async mount => await mount.shareWith(key));

    const getBucketKeys = async (id: string) => await tombMutex(async tomb => await tomb!.listBucketKeys(id));

    /** Returns list of snapshots for selected bucket */
    const getBucketShapshots = async (id: string) => await tombMutex(async tomb => await tomb!.listBucketSnapshots(id));

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

    /** Returns used storage amount in bytes */
    const getUsedStorage = async () => +(await tombMutex(async tomb => await tomb!.getUsage())).toString();

    /** Returns storage limit in bytes */
    const getUsageLimit = async () => +(await tombMutex(async tomb => await tomb!.getUsageLimit())).toString();

    const purgeSnapshot = async (id: string) => {
        // await tomb.purgeSnapshot(id);
    };
    /** Sets selected bucket into state */
    const selectBucket = async (bucket: Bucket) => {
        setSelectedBucket(bucket);
    };

    /** Returns selected bucket state according to current folder location. */
    const getSelectedBucketFiles = async (path: string[]) => {
        if (!selectedBucket) return

        const files = await selectedBucket?.mount.ls(path);
        setSelectedBucket(bucket => bucket ? ({ ...bucket, files }) : bucket);
    };

    /** Renames bucket */
    const moveTo = async (bucket: Bucket, from: string[], to: string[]) => {
        await mountMutex(bucket, async mount => {
            await mount.mv(from, to);
        });
    };
    /** Creates directory inside selected bucket */
    const createDirectory = async (bucket: Bucket, path: string[]) => {
        await mountMutex(bucket, async mount => {
            await mount.mkdir(path);
        });
    };

    /** Uploads file to selected bucket/directory, updates buckets state */
    const uploadFile = async (id: string, path: string[], name: string, file: ArrayBuffer) => {
        const bucket = buckets.find(bucket => bucket.id == id);
        try {
            await bucket?.mount.add([...path, name], file);
            const files = await bucket?.mount.ls(path) || [];
            setBuckets(buckets => buckets.map(bucket => {
                if (bucket.id === id) {
                    return ({ ...bucket, files: files })
                }
                return bucket;
            }))
        } catch (error: any) {
            console.log('uploadError', error);
        }
    };

    const renameFile = async (id: string, path: string[], newPath: string[]) => {
        // await tomb!.rename(id, path, newPath);
    };

    /** Creates bucket snapshot */
    const takeColdSnapshot = async (bucket: Bucket) => {
        await mountMutex(bucket, async mount => {
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
        const usedStorage = await getUsedStorage();
        const usageLimit = await getUsageLimit();
        setUsedStorage(usedStorage);
        setUsageLimit(usageLimit);
    };

    const deleteFile = async (bucket: Bucket, path: string[]) => {
        await mountMutex(bucket, async mount => {
            await mount.rm(path);
        });
    };

    // Initialize the tomb client
    useEffect(() => {
        if (!keystoreInitialized || !session?.accountId) { return; }

        (async () => {
            try {
                const apiKey = await getApiKey();
                const TombWasm = (await import('tomb-wasm-experimental')).TombWasm;
                const tomb = new TombWasm(
                    apiKey,
                    session.accountId,
                    "http://127.0.0.1:3001"
                );
                setTomb(tomb);
            } catch (err) {
                console.error(err);
            }
        })();
    }, [keystoreInitialized, session?.accountId]);

    useEffect(() => {
        if (tomb) {
            (async () => {
                try {
                    await getBuckets();
                    const usedStorage = await getUsedStorage();
                    const usageLimit = await getUsageLimit();
                    setUsedStorage(usedStorage);
                    setUsageLimit(usageLimit);
                } catch (error: any) { };
            })();
        };
    }, [tomb]);

    return (
        <TombContext.Provider
            value={{
                tomb, buckets, trash, usedStorage, usageLimit, areBucketsLoading, isTrashLoading, selectedBucket,
                getBuckets, getBucketShapshots, createBucket, deleteBucket, selectBucket, getFile,
                getTrashBucket, takeColdSnapshot, getUsedStorage, createDirectory,
                uploadFile, renameFile, getBucketKeys, purgeSnapshot, getSelectedBucketFiles,
                removeBucketAccess, approveBucketAccess, getUsageLimit,
                shareWith, download, moveTo, restore, deleteFile
            }}
        >
            {children}
        </TombContext.Provider>
    );
};

export const useTomb = () => useContext(TombContext);
