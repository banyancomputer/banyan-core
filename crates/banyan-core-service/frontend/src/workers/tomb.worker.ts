import { expose } from "comlink";
import { TombWasm, WasmBucket, WasmMount, WasmSnapshot } from 'tomb-wasm-experimental';

import {
	BrowserObject, Bucket, BucketKey,
	BucketSnapshot,
} from '../app/types/bucket';
import { handleNameDuplication } from "@/app/utils/names";
import { prettyFingerprintApiKeyPem, sortByType } from "@/app/utils";
import { StorageUsageClient } from "@/api/storageUsage";
import { SnapshotsClient } from "@/api/snapshots";
import { StorageLimits, StorageUsage } from "@/entities/storage";

const storageUsageClient = new StorageUsageClient();
const snapshotsClient = new SnapshotsClient();

type ApiKey = {
    privatePem: string;
    publicPem: string;
};

export class TombWorkerState {
    public tomb: null| TombWasm = null;
    public buckets: Bucket[] = [];
    public selectedBucket: Bucket| null = null;
    public apiKey: ApiKey | null = null;
    public encryptionKey: ApiKey | null = null;
    public areBucketsLoading: boolean = false;
    public storageUsage: StorageUsage = new StorageUsage();
    public storageLimits: StorageLimits = new StorageLimits();
};

export class TombWorker {
    public state = new TombWorkerState();

    async mountTomb(apiKey: ApiKey, userId: string, url: string, encryptionKey: ApiKey){
		this.state.apiKey = apiKey;
		this.state.encryptionKey = encryptionKey

        const tomb = await new TombWasm(
            apiKey.privatePem,
            userId,
            url
        );

        this.state.tomb = tomb;
		self.postMessage('tomb');
    };

    /** Returns list of buckets. */
    async getBuckets () {
		this.state.areBucketsLoading = true;
		const wasm_buckets: WasmBucket[] = await this.state.tomb!.listBuckets();
		// if (getIsUserNew()) {
			// 	createBucketAndMount("My Drive", 'hot', 'interactive');
            // 	destroyIsUserNew();
            // 	return;
            // }
            const buckets: Bucket[] = [];

            for (let bucket of wasm_buckets) {
				let mount;
                let locked;
                let isSnapshotValid;
                const snapshots = await snapshotsClient.getSnapshots(bucket.id());
                mount = await this.state.tomb!.mount(bucket.id(), this.state.encryptionKey!.privatePem);
                locked = await mount.locked();
                isSnapshotValid = await mount.hasSnapshot();
                buckets.push({
					mount: mount || null,
                    id: bucket.id(),
                    name: bucket.name(),
                    storageClass: bucket.storageClass(),
                    bucketType: bucket.bucketType(),
                    files: [],
                    snapshots,
                    keys: [],
                    locked: locked || false,
                    isSnapshotValid: isSnapshotValid || false
                });
            };
            this.state.buckets = buckets;
            this.state.areBucketsLoading = false;
			self.postMessage('buckets');
	};
		
	async remountBucket(bucketId: string) {
		const mount = await this.state.tomb!.mount(bucketId, this.state.encryptionKey!.privatePem);
		const locked = await mount.locked();
		const isSnapshotValid = await mount.hasSnapshot();
		this.state.buckets = this.state.buckets.map(element => element.id === bucketId ? { ...element, mount, locked, isSnapshotValid } : element);
	};

	/** Pushes keys inside of buckets list. */
	async getBucketsKeys () {
		this.state.areBucketsLoading = true;
		const buckets: Bucket[] = [];
		for (const bucket of this.state.buckets) {
			const rawKeys = await this.state.tomb!.listBucketKeys(bucket.id);
			const keys: BucketKey[] = [];
			for (let key of rawKeys) {
				const pem = key.publicKey;
				const approved = key.approved;
				const id = key.id;
				const fingerPrint = await prettyFingerprintApiKeyPem(pem);
				keys.push({ approved, bucket_id: bucket.id, fingerPrint, id, pem });
			};
			buckets.push({
				...bucket,
				keys,
			});
		}
		this.state.buckets = buckets;
		this.state.areBucketsLoading = false;
		self.postMessage('buckets');
	};

	public async getFiles (mount: WasmMount, path: string[]) {
		const rawFiles = await mount?.ls(path);
		return rawFiles ? rawFiles?.map(file => ({
			name: file.name,
			files: file.files || [],
			metadata: file.metadata,
			type: file.type
		})).sort(sortByType) : [];
	};

	/** Returns selected bucket state according to current folder location. */
	async getSelectedBucketFiles (path: string[]) {
		this.state.areBucketsLoading = true;
		const files = await this.getFiles(this.state.selectedBucket?.mount!, path);
		this.state.selectedBucket!.files = files;
		self.postMessage('selectedBucket');

		return files as BrowserObject[];
	};

    /** Sets selected bucket into state */
	selectBucket(bucketId: string | null) {
		const selectedBucket = this.state.buckets.find(bucket => bucket.id === bucketId);
		this.state.selectedBucket = selectedBucket || null;
		self.postMessage('selectedBucket');
	};

    /** Internal function which looking for selected bucket and updates it, or bucket in buckets list if no bucket selected. */
	updateBucketsState = (key: 'keys' | 'files' | 'snapshots' | 'isSnapshotValid'| 'name', elements: BrowserObject[] | BucketSnapshot[] | boolean | string, id: string,) => {
		/** If we are on buckets list screen there is no selected bucket in state. */
		if (this.state.selectedBucket?.id === id) {
			this.state.selectedBucket = {...this.state.selectedBucket, [key]: elements};
			self.postMessage('selectedBucket');
		};

		this.state.buckets = this.state.buckets.map(bucket => {
			if (bucket.id === id) {
				return { ...bucket, [key]: elements };
			};

			return bucket;
		});
		self.postMessage('buckets');
	};

	/** Uploads file to selected bucket/directory, updates buckets state */
	async uploadFile (bucketId: string, uploadPath: string[], folderLocation: string[], name: string, file: ArrayBuffer) {
		const mount = this.state.buckets.find(bucket => bucket.id === bucketId)?.mount!;
		const extstingFiles = (await this.getFiles(mount, uploadPath)).map(file => file.name);
		let fileName = handleNameDuplication(name, extstingFiles);
		await mount.write([...uploadPath, fileName], file);

		if (uploadPath.join('') !== folderLocation.join('')) { return; }
		const files = await this.getFiles(mount, uploadPath);
		await this.updateBucketsState('files', files, bucketId);
		const isSnapshotValid = await mount.hasSnapshot();
		await this.updateBucketsState('isSnapshotValid', isSnapshotValid, bucketId);
		await this.updateStorageUsageState();
	};

    async updateStorageUsageState  () {
		try {
			const usage = await storageUsageClient.getStorageUsage();
			this.state.storageUsage = usage;
		} catch (error: any) { };
	};

    /** Creates new bucket with recieved parameters of type and storag class. */
	async createBucketAndMount (name: string, storageClass: string, bucketType: string) {
			const { bucket: wasmBucket, mount: wasmMount } = await this.state.tomb!.createBucketAndMount(name, storageClass, bucketType, this.state.encryptionKey!.privatePem, this.state.encryptionKey!.publicPem);
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
			this.state.buckets = [...this.state.buckets, bucket].sort((a, b) => a.name.localeCompare(b.name));
			self.postMessage('buckets');

			return bucket.id
	};

    /** Returns file as ArrayBuffer */
	async getFile (bucketId: string, path: string[], name: string) {
		const bucket = this.state.buckets.find(bucket => bucket.id === bucketId)!;

		return await  bucket.mount!.readBytes([...path, name]);
	};

	/** Downloads file. */
	async download (bucketId: string, path: string[], name: string) {
        const bucket = this.state.buckets.find(bucket => bucket.id === bucketId)!;
		const arrayBuffer: Uint8Array = await this.getFile(bucket.id, path, name);

        return arrayBuffer;
	};

    /** Creates copy of fie in same direction with "Copy of" prefix. */
	async makeCopy (bucketId: string, path: string[], folderLocation: string[], name: string) {
        const arrayBuffer: ArrayBuffer = await this.getFile(bucketId, path, name);
		await this.uploadFile(bucketId, path, folderLocation, `Copy of ${name}`, arrayBuffer);
	};

    /** Restores bucket from selected snapshot. */
	async restore (bucketId: string, snapshotId: string) {
        await snapshotsClient.restoreFromSnapshot(bucketId, snapshotId);
	};

	/** Approves access key for bucket */
	async approveBucketAccess (bucketId: string, bucketKeyId: string) {
        const bucket = this.state.buckets.find(bucket => bucket.id === bucketId)!;
		await bucket.mount!.shareWith(bucketKeyId);
		await this.getBucketsKeys();
	};

    /** Returns list of snapshots for selected bucket */
	async getBucketSnapshots (id: string) {
		return await snapshotsClient.getSnapshots(id);
	};

    /** Approves a new deviceKey */
	async approveDeviceApiKey (pem: string) {
		await this.state.tomb!.approveDeviceApiKey(pem);
	};

    /** Deletes access key for bucket */
	async removeBucketAccess (bucketId: string, bucketKeyId: string) {
		/** TODO:  connect removeBucketAccess method when in will be implemented.  */
		await this.getBucketsKeys();
	};

    /** Changes placement of file inside bucket tree scructure */
	async moveTo (bucketId: string, from: string[], to: string[], name: string) {
		const bucket = this.state.buckets.find(bucket => bucket.id === bucketId)!;
		const extstingFiles = (await this.getFiles(bucket.mount!, to)).map(file => file.name);
		const browserObjectName = handleNameDuplication(name, extstingFiles);
		await bucket.mount!.mv(from, [...to, browserObjectName]);
		const isSnapshotValid = await bucket.mount!.hasSnapshot();
		await this.updateBucketsState('isSnapshotValid', isSnapshotValid, bucket.id);
	};

    /** Changes name of selected bucket. */
    async renameBucket (bucketId: string, newName: string) {
		const bucket = this.state.buckets.find(bucket => bucket.id === bucketId)!;
		await bucket.mount!.rename(newName);
		await this.updateBucketsState('name', newName, bucket.id);
		this.state.buckets = this.state.buckets.map(element => element.id === bucket.id ? { ...element, name: newName } : element);
	};

    /** Creates directory inside selected bucket */
	async createDirectory (bucketId: string, path: string[], name: string, folderLocation: string[]) {
		const bucket = this.state.buckets.find(bucket => bucket.id === bucketId)!;
		await bucket.mount!.mkdir([...path, name]);
		if (path.join('') !== folderLocation.join('')) { return; }
		const files = await this.getFiles(bucket.mount!, path) || [];
		await this.updateBucketsState('files', files.sort(sortByType), bucket.id);
		const isSnapshotValid = await bucket.mount!.hasSnapshot();
		await this.updateBucketsState('isSnapshotValid', isSnapshotValid, bucket.id);
		self.postMessage('selectedBucket');
	};

    async updateStorageLimitsState() {
		try {
			const limits = await storageUsageClient.getStorageLimits();
			this.state.storageLimits = limits;
		} catch (error: any) { };
	};

    /** Creates bucket snapshot */
	async takeColdSnapshot (bucketId: string) {
		const bucket = this.state.buckets.find(bucket => bucket.id === bucketId)!;
		await bucket.mount!.snapshot();
		const snapshots = await this.state.tomb!.listBucketSnapshots(bucket.id);
		await this.updateBucketsState('snapshots', snapshots, bucket.id);
		const isSnapshotValid = await bucket.mount!.hasSnapshot();
		await this.updateBucketsState('isSnapshotValid', isSnapshotValid, bucket.id);
	};

	async deleteBucket (id: string) {
		await this.state.tomb?.deleteBucket(id);
		await this.getBuckets();
		await this.updateStorageUsageState();
	};

	async deleteFile (bucketId: string, path: string[], name: string) {
		const bucket = this.state.buckets.find(bucket => bucket.id === bucketId)!;
			await bucket.mount!.rm([...path, name]);
			const isSnapshotValid = await bucket.mount!.hasSnapshot();
			await this.updateBucketsState('isSnapshotValid', isSnapshotValid, bucket.id);
    };
};

const worker = new TombWorker();

expose(worker);

self.postMessage('configured');