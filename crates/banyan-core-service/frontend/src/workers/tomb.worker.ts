import { expose } from "comlink";
import { TombWasm, WasmBucket, WasmMount, WasmSnapshot } from 'tomb-wasm-experimental';
import { Mutex } from 'async-mutex';

import {
	BrowserObject, Bucket, BucketKey,
	BucketSnapshot,
} from '../app/types/bucket';
import { handleNameDuplication } from "@/app/utils/names";
import { UserData, prettyFingerprintApiKeyPem, sortByType } from "@/app/utils";

const mutex = new Mutex();
	/** Prevents rust recursion error. */
	const tombMutex = async <T,>(tomb: T, callback: (tomb: T) => Promise<any>) => {
		const release = await mutex.acquire();
		try {
			return await callback(tomb);
		} catch (error) {
			console.error('tombMutex', error);
		} finally {
			release();
		}
	};

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
				}

				return bucket;
			});
			self.postMessage('buckets');
		};

		/** Returns list of buckets. */
		async getBuckets(){
			tombMutex(this.state.tomb!, async tomb => {
				const wasm_buckets: WasmBucket[] = await tomb.listBuckets();
				const buckets: Bucket[] = [];
				for (let bucket of wasm_buckets) {
					let mount;
					let locked;
					let isSnapshotValid;

					try {
						mount = await tomb.mount(bucket.id(), this.state.encryptionKey!.privatePem);
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
            this.state.buckets = buckets;
			self.postMessage('buckets');
		});
    };

	/** Pushes files and snapshots inside of buckets list. */
	async getBucketsFiles() {
		tombMutex(this.state.tomb!, async tomb => {
			try {
				this.state.areBucketsLoading = true;
				const wasm_bukets: Bucket[] = [];
				for (const bucket of this.state.buckets) {
					const files: BrowserObject[] = bucket.mount ? await bucket.mount!.ls([]) : [];
					const snapshots = await tomb.listBucketSnapshots(bucket.id);
					wasm_bukets.push({
						...bucket,
						snapshots,
						files,
					});
				};
				this.state.buckets = wasm_bukets;
				this.state.areBucketsLoading = false;
				self.postMessage('buckets');
			}catch(error: any) {
				console.log('error', error);
			}
		});
	};

	/** Pushes keys inside of buckets list. */
	async getBucketsKeys() {
		tombMutex(this.state.tomb!, async tomb => {
			this.state.areBucketsLoading = true;
			const wasm_bukets: Bucket[] = [];
			for (const bucket of this.state.buckets) {
				const rawKeys = await tomb.listBucketKeys(bucket.id);
				const keys: BucketKey[] = [];
				for (let key of rawKeys) {
					const pem = key.pem();
					const approved = key.approved();
					const id = key.id();
					const fingerPrint = await prettyFingerprintApiKeyPem(pem);
					keys.push({ approved, bucket_id: bucket.id, fingerPrint, id, pem })
					wasm_bukets.push({ ...bucket, keys });
					this.state.buckets = wasm_bukets;
				};
			}
			this.state.areBucketsLoading = false;
			self.postMessage('buckets');
		});
	};

	/** Sets selected bucket into state */
	selectBucket(bucketId: string | null) {
		const selectedBucket = this.state.buckets.find(bucket => bucket.id === bucketId);
		this.state.selectedBucket = selectedBucket || null;
		self.postMessage('selectedBucket');
	};

	/** Returns selected bucket state according to current folder location. */
	async getSelectedBucketFiles (path: string[]) {
		tombMutex(this.state.selectedBucket!.mount!, async mount => {
			const files = await mount.ls(path);
			this.state.selectedBucket = this.state.selectedBucket ? {...this.state.selectedBucket, files: files.sort(sortByType)}: null;
			self.postMessage('selectedBucket');
			return files;
		});
	};

	/** Returns selected bucket folders state according to current folder location. */
	async getSelectedBucketFolders (bucketId: string, path: string[]) {
		const bucket = this.state.buckets.find(bucket => bucket.id === bucketId)!;

		return tombMutex(bucket.mount!, async mount => {
			const files = await mount.ls(path);

			return files!.filter(browserObject => browserObject.type === 'dir');
		})
	}

	/** Uploads file to selected bucket/directory, updates buckets state. */
    async uploadFile(bucketId: string, uploadPath: string[], currentLocation: string[], name: string, file: ArrayBuffer, folder?: BrowserObject) {
        const mount = this.state.buckets.find(bucket => bucket.id === bucketId)!.mount!;

		tombMutex(mount, async mount => {
			const extstingFiles = (await mount.ls(uploadPath)).map(file => file.name);
			let fileName = handleNameDuplication(name, extstingFiles);
			await mount.write([...uploadPath, fileName], file);
			if (uploadPath.join('') !== currentLocation.join('')) { return; }
			const files = await mount.ls(uploadPath) || [];
			await this.updateBucketsState('files', files.sort(sortByType), bucketId);
			const isSnapshotValid = await mount.hasSnapshot();
			await this.updateBucketsState('isSnapshotValid', isSnapshotValid, bucketId);
			self.postMessage('selectedBucket');
		});
	};

	/** Creates new bucket with recieved parameters of type and storag class. */
	async createBucketAndMount (name: string, storageClass: string, bucketType: string) {
		return await tombMutex(this.state.tomb!, async tomb => {
			const { bucket: wasmBucket, mount: wasmMount } = await tomb!.createBucketAndMount(name, storageClass, bucketType, this.state.encryptionKey!.privatePem, this.state.encryptionKey!.publicPem);
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
		});
	};

	/** Returns file as ArrayBuffer */
	async getFile (bucketId: string, path: string[], name: string) {
		const bucket = this.state.buckets.find(bucket => bucket.id === bucketId)!;

		return await tombMutex(bucket.mount, async mount => await mount!.readBytes([...path, name]));
	};

	/** Restores buckets state to snapshot vesnion. */
	async restore (bucketId: string, snapshot: WasmSnapshot) {
		const bucket = this.state.buckets.find(bucket => bucket.id === bucketId)!;

		await tombMutex(bucket.mount!, async mount => await mount.restore(snapshot));
	};

	/** Generates public link to share file. */
	async shareFile (bucketId: string, path: string[]) {
		const bucket = this.state.buckets.find(bucket => bucket.id === bucketId)!;

		return await tombMutex(bucket.mount!, async mount => await mount.shareFile(path));
	};

	/** Approves access key for bucket */
	async approveBucketAccess (bucketId: string, bucket_key_id: string) {
		const bucket = this.state.buckets.find(bucket => bucket.id === bucketId)!;
		await tombMutex(bucket.mount!, async mount => {
			await mount.shareWith(bucket_key_id);
		});
		await this.getBucketsKeys();
	};

	/** Returns list of snapshots for selected bucket */
	async getBucketShapshots (id: string) {
		const rawSnapshots = await tombMutex(this.state.tomb!, async tomb => await tomb!.listBucketSnapshots(id));

		return rawSnapshots.map((snapshot:any) => ({ id: snapshot.id, bucket_id: snapshot.bucketId, snapshot_type: snapshot.snapshotType, version: snapshot.version, size: snapshot.size, createdAt: snapshot.createdAt }));
	};

	/** Approves a new deviceKey */
	async approveDeviceApiKey (pem: string) {
		await tombMutex(this.state.tomb!, async tomb => await tomb!.approveDeviceApiKey(pem));
	};

	/** Changes placement of file inside bucket tree scructure */
	async moveTo (bucketId: string, from: string[], to: string[], name: string) {
		const bucket = this.state.buckets.find(bucket => bucket.id === bucketId)!;
		await tombMutex(bucket.mount!, async mount => {
			const extstingFiles = (await mount.ls(to)).map(file => file.name);
			const browserObjectName = handleNameDuplication(name, extstingFiles);
			await mount.mv(from, [...to, browserObjectName]);
			const isSnapshotValid = await mount.hasSnapshot();
			await this.updateBucketsState('isSnapshotValid', isSnapshotValid, bucket.id);
		});
	};

	async renameBucket (bucketId: string, newName: string) {
		const bucket = this.state.buckets.find(bucket => bucket.id === bucketId)!;
		await tombMutex(bucket.mount!, async mount => {
			await mount.rename(newName);
			await this.updateBucketsState('name', newName, bucket.id);
			this.state.buckets = this.state.buckets.map(element => element.id === bucket.id ? { ...element, name: newName } : element);
		});
	};

	/** Creates directory inside selected bucket */
	async createDirectory (bucketId: string, path: string[], name: string, folderLocation: string[]) {
		const bucket = this.state.buckets.find(bucket => bucket.id === bucketId)!;
		await tombMutex(bucket.mount!, async mount => {
			await mount.mkdir([...path, name]);
			if (path.join('') !== folderLocation.join('')) { return; }
			const files = await mount.ls(path) || [];
			await this.updateBucketsState('files', files.sort(sortByType), bucket.id);
			const isSnapshotValid = await mount.hasSnapshot();
			await this.updateBucketsState('isSnapshotValid', isSnapshotValid, bucket.id);
			self.postMessage('selectedBucket');
		});
	};

	/** Returns used storage amount in bytes */
	async getStorageUsageState () {
		return await tombMutex(this.state.tomb!, async tomb => {
			const current = await tomb!.getUsage();
			const limit = await tomb!.getUsageLimit();

			return { current: Number(current), limit: Number(limit) };
		});
	};

	/** Creates bucket snapshot */
	async takeColdSnapshot (bucketId: string) {
		const bucket = this.state.buckets.find(bucket => bucket.id === bucketId)!;
		await tombMutex(this.state.tomb, async tomb => {
			await bucket.mount!.snapshot();
			const snapshots = await tomb!.listBucketSnapshots(bucket.id);
			await this.updateBucketsState('snapshots', snapshots, bucket.id);
			const isSnapshotValid = await bucket.mount!.hasSnapshot();
			await this.updateBucketsState('isSnapshotValid', isSnapshotValid, bucket.id);
		});
	};

	async deleteBucket (id: string) {
		await this.state.tomb?.deleteBucket(id);
		await this.getBuckets();
		await this.getStorageUsageState();
	};

	async deleteFile (bucketId: string, path: string[], name: string) {
		const bucket = this.state.buckets.find(bucket => bucket.id === bucketId)!;
		await tombMutex(bucket.mount!, async mount => {
			await mount.rm([...path, name]);
			const isSnapshotValid = await mount.hasSnapshot();
			await this.updateBucketsState('isSnapshotValid', isSnapshotValid, bucket.id);
		});
	};
};

const worker = new TombWorker();

expose(worker);