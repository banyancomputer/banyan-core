import { BrowserObject } from "@/app/types/bucket";
import { sortByType } from "@/app/utils";
import { expose } from "comlink";
import { TombWasm, WasmMount } from 'tomb-wasm-experimental';

type ApiKey = {
    privatePem: string;
    publicPem: string;
};

export class UploadWorkerState {
    public tomb: null| TombWasm = null;
    public buckets: {id: string, mount: WasmMount}[] = [];
    public apiKey: ApiKey | null = null;
    public encryptionKey: ApiKey | null = null;
};

export class UploadWorker {
    public state = new UploadWorkerState();

    async mountTomb(apiKey: ApiKey, userId: string, url: string, encryptionKey: ApiKey){
		this.state.apiKey = apiKey;
		this.state.encryptionKey = encryptionKey

        const tomb = await new TombWasm(
            apiKey.privatePem,
            userId,
            url
        );

        this.state.tomb = tomb;
    };

	/** Uploads file to selected bucket/directory, updates buckets state */
	async uploadFile (bucketId: string, uploadPath: string[], name: string, file: ArrayBuffer):Promise<BrowserObject[]> {
        let mount;
        const currentBucket = this.state.buckets.find(bucket => bucket.id === bucketId);
        if(currentBucket) {
            mount = currentBucket.mount
            console.error('oldMount');
        } else {
            console.error('newMount');
            mount = await this.state.tomb!.mount(bucketId, this.state.encryptionKey!.privatePem);
            this.state.buckets.push({mount, id: bucketId});
        };
		await mount.write([...uploadPath, name], file);
        // const rawFiles = await mount?.ls(uploadPath);
		return []
	};
};

const worker = new UploadWorker();

expose(worker);

self.postMessage('configured');