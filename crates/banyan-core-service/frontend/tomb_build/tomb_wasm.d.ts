/* tslint:disable */
/* eslint-disable */
/**
*/
export function setPanicHook(): void;
/**
*/
export class IntoUnderlyingByteSource {
  free(): void;
/**
* @param {any} controller
*/
  start(controller: any): void;
/**
* @param {any} controller
* @returns {Promise<any>}
*/
  pull(controller: any): Promise<any>;
/**
*/
  cancel(): void;
/**
*/
  readonly autoAllocateChunkSize: number;
/**
*/
  readonly type: string;
}
/**
*/
export class IntoUnderlyingSink {
  free(): void;
/**
* @param {any} chunk
* @returns {Promise<any>}
*/
  write(chunk: any): Promise<any>;
/**
* @returns {Promise<any>}
*/
  close(): Promise<any>;
/**
* @param {any} reason
* @returns {Promise<any>}
*/
  abort(reason: any): Promise<any>;
}
/**
*/
export class IntoUnderlyingSource {
  free(): void;
/**
* @param {any} controller
* @returns {Promise<any>}
*/
  pull(controller: any): Promise<any>;
/**
*/
  cancel(): void;
}
/**
* Raw options for [`pipeTo()`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/pipeTo).
*/
export class PipeOptions {
  free(): void;
/**
*/
  readonly preventAbort: boolean;
/**
*/
  readonly preventCancel: boolean;
/**
*/
  readonly preventClose: boolean;
/**
*/
  readonly signal: AbortSignal | undefined;
}
/**
*/
export class QueuingStrategy {
  free(): void;
/**
*/
  readonly highWaterMark: number;
}
/**
* Raw options for [`getReader()`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/getReader).
*/
export class ReadableStreamGetReaderOptions {
  free(): void;
/**
*/
  readonly mode: any;
}
/**
*/
export class TombWasm {
  free(): void;
/**
* Create a new TombWasm instance
* # Arguments
*
* * `web_signing_key` - The CryptoKeyPair to use for signing requests
* * `account_id` - The id of the account to use
* * `core_endpoint` - The API endpoint to use for core
* * `data_endpoint` - The API endpoint to use for data
*
* # Returns
*
* A new TombWasm instance
*
* Don't call it from multiple threads in parallel!
* @param {any} web_signing_key
* @param {string} account_id
* @param {string} core_endpoint
* @param {string} data_endpoint
*/
  constructor(web_signing_key: any, account_id: string, core_endpoint: string, data_endpoint: string);
/**
* Get the total consume storage space for the current account in bytes
* @returns {Promise<bigint>}
*/
  getUsage(): Promise<bigint>;
/**
* Get the current usage limit for the current account in bytes
* @returns {Promise<bigint>}
*/
  getUsageLimit(): Promise<bigint>;
/**
* List the buckets for the current account
* @returns {Promise<Array<any>>}
*/
  listBuckets(): Promise<Array<any>>;
/**
* List bucket snapshots for a bucket
*
* # Arguments
*
* * `bucket_id` - The id of the bucket to list snapshots for
*
* # Returns an array WasmSnapshots
*
* ```json
* [
*   {
*     "id": "ffc1dca2-5155-40be-adc6-c81eb7322fb8",
*     "bucket_id": "f0c55cc7-4896-4ff3-95de-76422af271b2",
*     "metadata_id": "05d063f1-1e3f-4876-8b16-aeb106af0eb0",
*     "created_at": "2023-09-05T19:05:34Z"
*   }
* ]
* ```
* @param {string} bucket_id
* @returns {Promise<Array<any>>}
*/
  listBucketSnapshots(bucket_id: string): Promise<Array<any>>;
/**
* List bucket keys for a bucket
* # Arguments
* * `bucket_id` - The id of the bucket to list keys for
* # Returns an array of WasmBucketKeys in the form:
* ```json
* [
* {
* "id": "uuid",
* "bucket_id": "uuid",
* "pem": "string"
* "approved": "bool"
* }
* ]
* ```
* @param {string} bucket_id
* @returns {Promise<Array<any>>}
*/
  listBucketKeys(bucket_id: string): Promise<Array<any>>;
/**
* Create a new bucket
* # Arguments
* * `name` - The name of the bucket to create
* * `storage_class` - The storage class of the bucket to create
* * `bucket_type` - The type of the bucket to create
* * `encryption_key` - The encryption key to use for the bucket
* # Returns
* The bucket's metadata as a WasmBucket
* ```json
* {
* "id": "uuid",
* "name": "string"
* "type": "string",
* "storage_class": "string",
* }
* ```
* @param {string} name
* @param {string} storage_class
* @param {string} bucket_type
* @param {CryptoKey} initial_key
* @returns {Promise<WasmBucket>}
*/
  createBucket(name: string, storage_class: string, bucket_type: string, initial_key: CryptoKey): Promise<WasmBucket>;
/**
* Create a bucket key for a bucket
* # Arguments
* * `bucket_id` - The id of the bucket to create a key for
* # Returns
* The WasmBucketKey that was created
* @param {string} bucket_id
* @returns {Promise<WasmBucketKey>}
*/
  createBucketKey(bucket_id: string): Promise<WasmBucketKey>;
/**
* Delete a bucket
* # Arguments
* * `bucket_id` - The id of the bucket to delete
* # Returns the id of the bucket that was deleted
* @param {string} bucket_id
* @returns {Promise<void>}
*/
  deleteBucket(bucket_id: string): Promise<void>;
/**
* Mount a bucket as a File System that can be managed by the user
* # Arguments
* * bucket_id - The id of the bucket to mount
* * key - The key to use to mount the bucket. This should be the crypto key pair that was used to create the bucket
*         or that has access to the bucket
* # Returns
* A WasmMount instance
* @param {string} bucket_id
* @param {any} key
* @returns {Promise<WasmMount>}
*/
  mount(bucket_id: string, key: any): Promise<WasmMount>;
}
/**
*/
export class WasmBucket {
  free(): void;
/**
* @returns {string}
*/
  bucketType(): string;
/**
* @returns {string}
*/
  id(): string;
/**
* @returns {string}
*/
  name(): string;
/**
* @returns {string}
*/
  storageClass(): string;
}
/**
*/
export class WasmBucketKey {
  free(): void;
/**
* @returns {boolean}
*/
  approved(): boolean;
/**
* @returns {string}
*/
  bucketId(): string;
/**
* @returns {string}
*/
  id(): string;
/**
* @returns {string}
*/
  pem(): string;
}
/**
* A wrapper around a bucket metadata
*/
export class WasmBucketMetadata {
  free(): void;
/**
*/
  readonly bucketId: string;
/**
*/
  readonly id: string;
/**
*/
  readonly snapshotId: string;
}
/**
* Mount point for a Bucket in WASM
*
* Enables to call Fs methods on a Bucket, pulling metadata from a remote
*/
export class WasmMount {
  free(): void;
/**
* Returns whether or not the bucket is dirty (this will be true when a file or directory has
* been changed).
* @returns {boolean}
*/
  dirty(): boolean;
/**
* Returns whether or not the bucket is locked
* @returns {boolean}
*/
  locked(): boolean;
/**
* Returns the Metadata for the bucket
* @returns {WasmBucketMetadata}
*/
  metadata(): WasmBucketMetadata;
/**
* List the contents of the bucket at a provided path
*
* # Arguments
*
* * `path_segments` - The path to ls (as an Array)
*
* # Returns
*
* The an Array of objects in the form of:
*
* ```json
* [
*   {
*     "name": "string",
*     "entry_type": "(file | dir)"
*     "metadata": {
*       "created": 0,
*       "modified": 0,
*       "size": 0,
*       "cid": "string"
*     }
*   }
* ]
* ```
*
* # Errors
*
* * `Bucket is locked` - If the bucket is locked
* @param {Array<any>} path_segments
* @returns {Promise<Array<any>>}
*/
  ls(path_segments: Array<any>): Promise<Array<any>>;
/**
* Mkdir
* # Arguments
* * `path_segments` - The path to mkdir (as an Array)
* # Returns
* Promise<void> in js speak
* # Errors
* * `Bucket is locked` - If the bucket is locked
* * `Could not mkdir` - If the mkdir fails
* * `Could not sync` - If the sync fails
* @param {Array<any>} path_segments
* @returns {Promise<void>}
*/
  mkdir(path_segments: Array<any>): Promise<void>;
/**
* Write a file
* # Arguments
* * `path_segments` - The path to write to (as an Array)
* * `content_buffer` - The content to write (as an ArrayBuffer)
* # Returns
* Promise<void> in js speak
* # Errors
* * `Bucket is locked` - If the bucket is locked
* * `Could not add` - If the add fails
* * `Could not sync` - If the sync fails
* @param {Array<any>} path_segments
* @param {ArrayBuffer} content_buffer
* @returns {Promise<void>}
*/
  write(path_segments: Array<any>, content_buffer: ArrayBuffer): Promise<void>;
/**
* Read a file from a mounted bucket
*     Read / Download a File (takes a path to a file inside the bucket, not available for cold only buckets)
*     Allows reading at a version
* # Arguments
* * `path_segments` - The path to read from (as an Array)
* * `version` - The version to read from (optional)
* # Returns
* A Promise<ArrayBuffer> in js speak
* @param {Array<any>} path_segments
* @param {string | undefined} _version
* @returns {Promise<Uint8Array>}
*/
  readBytes(path_segments: Array<any>, _version?: string): Promise<Uint8Array>;
/**
* Mv a file or directory
* # Arguments
* * `from_path_segments` - The path to mv from (as an Array)
* * `to_path_segments` - The path to mv to (as an Array)
* # Returns
* Promise<void> in js speak
* # Errors
* * `Bucket is locked` - If the bucket is locked
* * `Could not mv` - If the mv fails, such as if the path does not exist in the bucket
* * `Could not sync` - If the sync fails
* @param {Array<any>} from_path_segments
* @param {Array<any>} to_path_segments
* @returns {Promise<void>}
*/
  mv(from_path_segments: Array<any>, to_path_segments: Array<any>): Promise<void>;
/**
* Rm a file or directory
* # Arguments
* * `path_segments` - The path to rm (as an Array)
* # Returns
* Promise<void> in js speak
* # Errors
* * `Bucket is locked` - If the bucket is locked
* * `Could not rm` - If the rm fails
* * `Could not sync` - If the sync fails
* @param {Array<any>} path_segments
* @returns {Promise<void>}
*/
  rm(path_segments: Array<any>): Promise<void>;
/**
* Share with
* # Arguments
* * bucket_key_id - The id of the bucket key to share with
* # Returns
* Promise<void> in js speak
* # Errors
* * `could not read bucket key` - If the bucket key cannot be read (such as if it does not exist, or does not belong to the bucket)
* * `Bucket is locked` - If the bucket is locked
* * `could not share with` - If the share fails
* @param {string} bucket_key_id
* @returns {Promise<void>}
*/
  shareWith(bucket_key_id: string): Promise<void>;
/**
* Return boolean indiciating whether or not the currently mounted bucket is snapshotted
* # Returns
* A boolean
* # Errors
* * "missing metadata" - If the metadata is missing
* @returns {boolean}
*/
  hasSnapshot(): boolean;
/**
* Snapshot a mounted bucket
* # Returns
* A Promise<void> in js speak
* # Errors
* * "missing metadata" - If the metadata is missing
* * "could not snapshot" - If the snapshot fails
* @returns {Promise<string>}
*/
  snapshot(): Promise<string>;
/**
* Restore a mounted bucket
* # Arguments
* * `wasm_snapshot` - The snapshot to restore from
* # Returns
* A Promise<void> in js speak. Should update the mount to the version of the snapshot
* @param {WasmSnapshot} wasm_snapshot
* @returns {Promise<void>}
*/
  restore(wasm_snapshot: WasmSnapshot): Promise<void>;
}
/**
*/
export class WasmSnapshot {
  free(): void;
/**
*/
  readonly bucketId: string;
/**
*/
  readonly createdAt: number;
/**
*/
  readonly id: string;
/**
*/
  readonly metadataId: string;
/**
*/
  readonly size: number;
}
