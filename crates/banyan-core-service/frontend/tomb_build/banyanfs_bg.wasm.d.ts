/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export function __wbg_wasmsnapshot_free(a: number): void;
export function wasmsnapshot_bucket_id(a: number, b: number): void;
export function wasmsnapshot_created_at(a: number): number;
export function wasmsnapshot_id(a: number, b: number): void;
export function wasmsnapshot_metadataId(a: number, b: number): void;
export function wasmsnapshot_size(a: number): number;
export function wasm_init(): void;
export function __wbg_wasmnodemetadata_free(a: number): void;
export function __wbg_wasmbucket_free(a: number): void;
export function wasmbucket_bucketType(a: number, b: number): void;
export function wasmbucket_id(a: number, b: number): void;
export function wasmbucket_name(a: number, b: number): void;
export function wasmbucket_storageClass(a: number, b: number): void;
export function wasmsharedfile_fileName(a: number, b: number): void;
export function wasmsharedfile_mimeType(a: number, b: number): void;
export function __wbg_wasmsharedfile_free(a: number): void;
export function __wbg_wasmbucketkey_free(a: number): void;
export function wasmbucketkey_approved(a: number): number;
export function wasmbucketkey_bucket_id(a: number, b: number): void;
export function wasmbucketkey_fingerprint(a: number, b: number): void;
export function wasmbucketkey_id(a: number, b: number): void;
export function wasmbucketkey_public_key(a: number, b: number): void;
export function __wbg_wasmbucketmetadata_free(a: number): void;
export function wasmbucketmetadata_bucket_id(a: number, b: number): void;
export function wasmbucketmetadata_id(a: number, b: number): void;
export function wasmbucketmetadata_snapshot_id(a: number, b: number): void;
export function __wbg_wasmbucketmount_free(a: number): void;
export function wasmbucketmount_bucket(a: number): number;
export function wasmbucketmount_mount(a: number): number;
export function wasmbucketmount_new(a: number, b: number): number;
export function __wbg_wasmfsmetadataentry_free(a: number): void;
export function wasmfsmetadataentry_metadata(a: number): number;
export function __wbg_wasmmount_free(a: number): void;
export function wasmmount_bucket(a: number): number;
export function wasmmount_dirty(a: number): number;
export function wasmmount_hasSnapshot(a: number): number;
export function wasmmount_locked(a: number): number;
export function wasmmount_ls(a: number, b: number): number;
export function wasmmount_metadata(a: number, b: number): void;
export function wasmmount_mkdir(a: number, b: number): number;
export function wasmmount_mv(a: number, b: number, c: number): number;
export function wasmmount_readBytes(a: number, b: number, c: number, d: number): number;
export function wasmmount_remount(a: number, b: number, c: number): number;
export function wasmmount_rename(a: number, b: number, c: number): number;
export function wasmmount_restore(a: number, b: number): number;
export function wasmmount_rm(a: number, b: number): number;
export function wasmmount_shareFile(a: number, b: number): number;
export function wasmmount_shareWith(a: number, b: number, c: number): number;
export function wasmmount_snapshot(a: number): number;
export function wasmmount_write(a: number, b: number, c: number): number;
export function __wbg_tombwasm_free(a: number): void;
export function tombwasm_approveDeviceApiKey(a: number, b: number, c: number): number;
export function tombwasm_createBucketAndMount(a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number): number;
export function tombwasm_createBucketKey(a: number, b: number, c: number): number;
export function tombwasm_deleteBucket(a: number, b: number, c: number): number;
export function tombwasm_getUsage(a: number): number;
export function tombwasm_getUsageLimit(a: number): number;
export function tombwasm_listBuckets(a: number): number;
export function tombwasm_listBucketKeys(a: number, b: number, c: number): number;
export function tombwasm_listBucketSnapshots(a: number, b: number, c: number): number;
export function tombwasm_mount(a: number, b: number, c: number, d: number, e: number): number;
export function tombwasm_new(a: number, b: number, c: number, d: number, e: number, f: number): number;
export function tombwasm_renameBucket(a: number, b: number, c: number, d: number, e: number): number;
export function wasmfsmetadataentry_entry_kind(a: number, b: number): void;
export function wasmfsmetadataentry_name(a: number, b: number): void;
export function __wbg_intounderlyingbytesource_free(a: number): void;
export function intounderlyingbytesource_type(a: number, b: number): void;
export function intounderlyingbytesource_autoAllocateChunkSize(a: number): number;
export function intounderlyingbytesource_start(a: number, b: number): void;
export function intounderlyingbytesource_pull(a: number, b: number): number;
export function intounderlyingbytesource_cancel(a: number): void;
export function __wbg_intounderlyingsource_free(a: number): void;
export function intounderlyingsource_pull(a: number, b: number): number;
export function intounderlyingsource_cancel(a: number): void;
export function __wbg_intounderlyingsink_free(a: number): void;
export function intounderlyingsink_write(a: number, b: number): number;
export function intounderlyingsink_close(a: number): number;
export function intounderlyingsink_abort(a: number, b: number): number;
export function __wbindgen_malloc(a: number, b: number): number;
export function __wbindgen_realloc(a: number, b: number, c: number, d: number): number;
export const __wbindgen_export_2: WebAssembly.Table;
export function _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h514f31ff2b841dc9(a: number, b: number, c: number): void;
export function _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h205286b1685d054d(a: number, b: number, c: number): void;
export function __wbindgen_add_to_stack_pointer(a: number): number;
export function __wbindgen_free(a: number, b: number, c: number): void;
export function __wbindgen_exn_store(a: number): void;
export function wasm_bindgen__convert__closures__invoke2_mut__h838a97cacb6f33e9(a: number, b: number, c: number, d: number): void;
export function __wbindgen_start(): void;
