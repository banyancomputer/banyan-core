import React, { ReactNode, createContext, useContext, useEffect, useState } from 'react';
import { TombWasm, WasmBucket, WasmMount } from 'tomb-wasm-experimental';
import { Mutex } from 'async-mutex';
import { useSession } from 'next-auth/react';

import { useKeystore } from './keystore';
import {
    Bucket, BucketFile, BucketKey,
    BucketSnapshot, Metadata, MockBucket,
} from '@/lib/interfaces/bucket';

interface TombInterface {
    tomb: TombWasm | null;
    buckets: WasmBucket[];
    mounts: Map<string, WasmMount>;
    usedStorage: number;
    usageLimit: number;
    trash: Bucket;
    isTrashLoading: boolean;
    areBucketsLoading: boolean;
    mountBucket: (id: string) => Promise<void>;
    download: (bucketId: string, path: string[]) => Promise<ArrayBuffer | undefined>;
    shareWith: (bucketId: string, key: string[]) => Promise<void>
    takeColdSnapshot: (id: string) => Promise<void>;
    getBuckets: () => Promise<void>;
    createBucket: (name: string, storageClass: string, bucketType: string) => Promise<void>;
    createDirectory: (bucketId: string, path: string[]) => Promise<void>;
    uploadFile: (id: string, path: string, file: any) => Promise<void>;
    renameFile: (id: string, path: string, newPath: string) => Promise<void>;
    getTrashBucket: () => Promise<void>;
    getUsedStorage: () => Promise<number>;
    getUsageLimit: () => Promise<number>;
    getBucketShapshots: (id: string) => Promise<BucketSnapshot[]>;
    getBucketKeys: (id: string) => Promise<BucketKey[]>;
    purgeSnapshot: (id: string) => void;
    deleteBucket: (id: string) => void;
    approveBucketAccess: (id: string) => Promise<void>;
    removeBucketAccess: (id: string) => Promise<void>;
};

const tombMutex = new Mutex();

const TombContext = createContext<TombInterface>({} as TombInterface);

export const TombProvider = ({ children }: { children: ReactNode }) => {
    // The active user's session
    const { data: session } = useSession();
    // The active user's keystore
    const { keystoreInitialized, getEncryptionKey, getApiKey } = useKeystore();
    const [tomb, setTomb] = useState<TombWasm | null>(null);
    const [buckets, setBuckets] = useState<WasmBucket[]>([]);
    const [mounts, setMounts] = useState<Map<string, WasmMount>>(new Map());
    const [trash, setTrash] = useState<Bucket>(new MockBucket());
    const [usedStorage, setUsedStorage] = useState<number>(0);
    const [usageLimit, setUsageLimit] = useState<number>(0);
    const [isTrashLoading, setIsTrashLoading] = useState<boolean>(true);
    const [areBucketsLoading, setAreBucketsLoading] = useState<boolean>(true);

    /** Prevents rust recursion error. */
    const mutex = async (calllack: (tomb: TombWasm) => Promise<any>) => {
        if (tomb) {
            const release = await tombMutex.acquire();
            try {
                return await calllack(tomb);
            } catch (err) {
                console.error('err', err);
            } finally {
                release();
            }
        }
    };

    /** Returns bucket abstraction.  */
    const mountBucket = async (id: string) => {
        await mutex(async tomb => {
            let key = await getEncryptionKey();
            return await tomb.mount(id, key);
        });
    };

    /** Returns list of buckets. */
    const getBuckets = async () => {
        setAreBucketsLoading(true);
        await mutex(async tomb => {
            const buckets = await tomb!.listBuckets().
                catch(err => {
                    console.error(err);
                    return [];
                });
            console.log('buckets', buckets);

            setBuckets(buckets);
        })
        setAreBucketsLoading(false);
    };

    /** Creates new bucket with recieved parameters of type and storag class. */
    const createBucket = async (name: string, storageClass: string, bucketType: string) => {
        await mutex(async tomb => {
            let key = await getEncryptionKey();
            console.log('creating bucket', name, storageClass, bucketType);
            let wasm_bucket = await tomb.createBucket(name, storageClass, bucketType, key.publicKey);
            console.log('wasm_bucket', wasm_bucket.id());
            console.log('wasm_bucket', wasm_bucket.name());
            console.log('mounting bucket');
            let mount = await tomb.mount(wasm_bucket.id(), key);
            console.log('mount', mount);
            setBuckets([...buckets, wasm_bucket]);
            let new_mounts = mounts.set(wasm_bucket.id(), mount);
            setMounts(new_mounts);
        })
    };

    /** Retuns array buffer of selected file. */
    const download = async (bucketId: string, path: string[]) => await mounts.get(bucketId)?.readBytes(path);

    const shareWith = async (bucketId: string, key: string) => await mounts.get(bucketId)?.shareWith(key);

    const getBucketKeys = async (id: string) => await mutex(async tomb => await tomb!.listBucketKeys(id));

    const deleteBucket = async (id: string) => await tomb!.deleteBucket(id);

    /** Returns list of snapshots for selected bucket */
    const getBucketShapshots = async (id: string) => await tomb!.listBucketSnapshots(id);

    /** Approves access key for bucket */
    const approveBucketAccess = async (id: string) => {
        /** TODO:  connect approveBucketAccess method when in will be implemented.  */
        // await tomb!.approveBucketAccess(id);
    };

    /** Deletes access key for bucket */
    const removeBucketAccess = async (id: string) => {
        /** TODO:  connect removeBucketAccess method when in will be implemented.  */
        // return await tomb!.approveBucketAccess(id);
    };

    /** Returns used storage amount in bytes */
    const getUsedStorage = async () => +(await tomb!.getUsage()).toString();

    /** Returns storage limit in bytes */
    const getUsageLimit = async () => +(await tomb!.getUsageLimit()).toString();

    const purgeSnapshot = async (id: string) => {
        // await tomb!.purgeSnapshot(id);
    };

    /** Creates directory inside selected bucket */
    const createDirectory = async (bucketId: string, path: string[]) => {
        await mounts.get(bucketId)?.mkdir(path);
    };

    const uploadFile = async (id: string, path: string, file: any) => {
        // await tomb!.upload(id, path, file);
    };

    const renameFile = async (id: string, path: string, newPath: string) => {
        // await tomb!.rename(id, path, newPath);
    };

    /** Creates bucket snapshot */
    const takeColdSnapshot = async (id: string) => {
        mounts.get(id)?.snapshot();
    }

    /** TODO: implement after adding to tomb-wasm */
    const getTrashBucket: () => Promise<void> = async () => {
        // setIsTrashLoading(true);
        // const trash = await tomb!();
        // const files = await getFiles(trash.id, '/');
        // setTrash({ ...trash, files });
        // setIsTrashLoading(false);
    }

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
                    "http://localhost:3001"
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
                tomb, buckets, trash, usedStorage, usageLimit, areBucketsLoading, isTrashLoading, mounts,
                getBuckets, getBucketShapshots, mountBucket, createBucket,
                getTrashBucket, takeColdSnapshot, getUsedStorage, createDirectory,
                uploadFile, renameFile, getBucketKeys, purgeSnapshot,
                removeBucketAccess, approveBucketAccess, deleteBucket, getUsageLimit,
                shareWith, download
            }}
        >
            {children}
        </TombContext.Provider>
    );
};

export const useTomb = () => useContext(TombContext);