/* tslint:disable */
/* eslint-disable */
/**
* Performs first time setup to the WASM environment once this library is loaded. This primarily
* sets up logging and reports the library version.
*/
export function wasm_init(): void;
/**
*/
export class IntoUnderlyingByteSource {
  free(): void;
/**
* @param {ReadableByteStreamController} controller
*/
  start(controller: ReadableByteStreamController): void;
/**
* @param {ReadableByteStreamController} controller
* @returns {Promise<any>}
*/
  pull(controller: ReadableByteStreamController): Promise<any>;
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
* @param {ReadableStreamDefaultController} controller
* @returns {Promise<any>}
*/
  pull(controller: ReadableStreamDefaultController): Promise<any>;
/**
*/
  cancel(): void;
}
/**
*/
export class TombWasm {
  free(): void;
/**
* @param {string} name
* @param {string} public_pem
* @returns {Promise<void>}
*/
  createUserKey(name: string, public_pem: string): Promise<void>;
/**
* @param {string} name
* @param {string} user_key_id
* @returns {Promise<void>}
*/
  renameUserKey(name: string, user_key_id: string): Promise<void>;
/**
* @param {string} bucket_id
* @param {string} fingerprint
* @returns {Promise<void>}
*/
  revokeBucketAccess(bucket_id: string, fingerprint: string): Promise<void>;
/**
* @returns {Promise<Array<any>>}
*/
  userKeyAccess(): Promise<Array<any>>;
/**
* @param {string} name
* @param {string} storage_class
* @param {string} bucket_type
* @param {string} private_key_pem
* @param {string} public_key_pem
* @returns {Promise<WasmBucketMount>}
*/
  createBucketAndMount(name: string, storage_class: string, bucket_type: string, private_key_pem: string, public_key_pem: string): Promise<WasmBucketMount>;
/**
* @param {string} bucket_id
* @returns {Promise<void>}
*/
  deleteBucket(bucket_id: string): Promise<void>;
/**
* @returns {Promise<number>}
*/
  getUsage(): Promise<number>;
/**
* @returns {Promise<number>}
*/
  getUsageLimit(): Promise<number>;
/**
* @returns {Promise<Array<any>>}
*/
  listBuckets(): Promise<Array<any>>;
/**
* @param {string} bucket_id
* @returns {Promise<Array<any>>}
*/
  listBucketAccess(bucket_id: string): Promise<Array<any>>;
/**
* @param {string} bucket_id
* @returns {Promise<Array<any>>}
*/
  listBucketSnapshots(bucket_id: string): Promise<Array<any>>;
/**
* @param {string} drive_id
* @param {string} private_key_pem
* @returns {Promise<WasmMount>}
*/
  mount(drive_id: string, private_key_pem: string): Promise<WasmMount>;
/**
* @param {string} private_key_pem
* @param {string} account_id
* @param {string} api_endpoint
*/
  constructor(private_key_pem: string, account_id: string, api_endpoint: string);
/**
* @param {string} bucket_id
* @param {string} name
* @returns {Promise<void>}
*/
  renameBucket(bucket_id: string, name: string): Promise<void>;
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
export class WasmBucketAccess {
  free(): void;
/**
*/
  readonly driveId: string;
/**
*/
  readonly fingerprint: string;
/**
*/
  readonly state: string;
/**
*/
  readonly userKeyId: string;
}
/**
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
*/
export class WasmBucketMount {
  free(): void;
/**
* @param {WasmBucket} bucket
* @param {WasmMount} mount
* @returns {WasmBucketMount}
*/
  static new(bucket: WasmBucket, mount: WasmMount): WasmBucketMount;
/**
*/
  readonly bucket: WasmBucket;
/**
*/
  readonly mount: WasmMount;
}
/**
*/
export class WasmFsMetadataEntry {
  free(): void;
/**
*/
  readonly metadata: any;
/**
*/
  readonly name: string;
/**
*/
  readonly type: string;
}
/**
*/
export class WasmMount {
  free(): void;
/**
* @returns {WasmBucket}
*/
  bucket(): WasmBucket;
/**
* @returns {boolean}
*/
  dirty(): boolean;
/**
* @returns {boolean}
*/
  hasSnapshot(): boolean;
/**
* @returns {boolean}
*/
  locked(): boolean;
/**
* @param {Array<any>} path_segments
* @returns {Promise<Array<any>>}
*/
  ls(path_segments: Array<any>): Promise<Array<any>>;
/**
* @returns {WasmBucketMetadata}
*/
  metadata(): WasmBucketMetadata;
/**
* @param {Array<any>} path_segments
* @returns {Promise<void>}
*/
  mkdir(path_segments: Array<any>): Promise<void>;
/**
* @param {Array<any>} src_path_segments
* @param {Array<any>} dst_path_segments
* @returns {Promise<void>}
*/
  mv(src_path_segments: Array<any>, dst_path_segments: Array<any>): Promise<void>;
/**
* @param {Array<any>} path_segments
* @param {string | undefined} [_version]
* @returns {Promise<Uint8Array>}
*/
  readBytes(path_segments: Array<any>, _version?: string): Promise<Uint8Array>;
/**
* @param {string} _key_pem
* @returns {Promise<void>}
*/
  remount(_key_pem: string): Promise<void>;
/**
* @param {string} name
* @returns {Promise<void>}
*/
  rename(name: string): Promise<void>;
/**
* @param {WasmSnapshot} wasm_snapshot
* @returns {Promise<void>}
*/
  restore(wasm_snapshot: WasmSnapshot): Promise<void>;
/**
* @param {Array<any>} path_segments
* @returns {Promise<void>}
*/
  rm(path_segments: Array<any>): Promise<void>;
/**
* @param {Array<any>} _path_segments
* @returns {Promise<string>}
*/
  shareFile(_path_segments: Array<any>): Promise<string>;
/**
* @param {string} _bucket_key_id
* @returns {Promise<void>}
*/
  shareWith(_bucket_key_id: string): Promise<void>;
/**
* @returns {Promise<string>}
*/
  snapshot(): Promise<string>;
/**
* @param {Array<any>} path_segments
* @param {ArrayBuffer} content_buffer
* @returns {Promise<void>}
*/
  write(path_segments: Array<any>, content_buffer: ArrayBuffer): Promise<void>;
}
/**
*/
export class WasmNodeMetadata {
  free(): void;
}
/**
*/
export class WasmSharedFile {
  free(): void;
/**
* @returns {string}
*/
  fileName(): string;
/**
* @returns {string}
*/
  mimeType(): string;
}
/**
*/
export class WasmSnapshot {
  free(): void;
/**
* @returns {string}
*/
  id(): string;
/**
* @returns {string}
*/
  metadataId(): string;
/**
*/
  readonly bucketId: string;
/**
*/
  readonly createdAt: bigint;
/**
*/
  readonly size: bigint;
}
/**
*/
export class WasmUserKey {
  free(): void;
/**
* Key Id
* @returns {string}
*/
  id(): string;
/**
* Name of the Key
* @returns {string}
*/
  name(): string;
/**
* User Id of the Owner of the Key
* @returns {string}
*/
  userId(): string;
/**
* API usability
* @returns {boolean}
*/
  apiAccess(): boolean;
/**
* Public Key PEM
* @returns {string}
*/
  public_key_pem(): string;
/**
* Public Key Fingerprint
* @returns {string}
*/
  fingerprint(): string;
/**
* Created at timestamp
* @returns {string}
*/
  createdAt(): string;
}
/**
*/
export class WasmUserKeyAccess {
  free(): void;
/**
*/
  readonly bucketIds: Array<any>;
/**
*/
  readonly key: WasmUserKey;
}
