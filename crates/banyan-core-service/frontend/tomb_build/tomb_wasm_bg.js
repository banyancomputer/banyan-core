let wasm;
export function __wbg_set_wasm(val) {
    wasm = val;
}


const heap = new Array(128).fill(undefined);

heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }

let heap_next = heap.length;

function dropObject(idx) {
    if (idx < 132) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

const lTextDecoder = typeof TextDecoder === 'undefined' ? (0, module.require)('util').TextDecoder : TextDecoder;

let cachedTextDecoder = new lTextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachedUint8Memory0 = null;

function getUint8Memory0() {
    if (cachedUint8Memory0 === null || cachedUint8Memory0.byteLength === 0) {
        cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

let WASM_VECTOR_LEN = 0;

const lTextEncoder = typeof TextEncoder === 'undefined' ? (0, module.require)('util').TextEncoder : TextEncoder;

let cachedTextEncoder = new lTextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachedInt32Memory0 = null;

function getInt32Memory0() {
    if (cachedInt32Memory0 === null || cachedInt32Memory0.byteLength === 0) {
        cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32Memory0;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_2.get(state.dtor)(a, state.b);

            } else {
                state.a = a;
            }
        }
    };
    real.original = state;

    return real;
}
function __wbg_adapter_32(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures__invoke1_mut__h1e3a2769c21eb913(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_35(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h5444dea59ff3ad65(arg0, arg1, addHeapObject(arg2));
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
    return instance.ptr;
}
/**
*/
export function setPanicHook() {
    wasm.setPanicHook();
}

let cachedUint32Memory0 = null;

function getUint32Memory0() {
    if (cachedUint32Memory0 === null || cachedUint32Memory0.byteLength === 0) {
        cachedUint32Memory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32Memory0;
}

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getUint32Memory0();
    const slice = mem.subarray(ptr / 4, ptr / 4 + len);
    const result = [];
    for (let i = 0; i < slice.length; i++) {
        result.push(takeObject(slice[i]));
    }
    return result;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_exn_store(addHeapObject(e));
    }
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8Memory0().subarray(ptr / 1, ptr / 1 + len);
}
function __wbg_adapter_250(arg0, arg1, arg2, arg3) {
    wasm.wasm_bindgen__convert__closures__invoke2_mut__ha21e6559f50fa7a5(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
}

/**
*/
export class IntoUnderlyingByteSource {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_intounderlyingbytesource_free(ptr);
    }
    /**
    * @returns {string}
    */
    get type() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.intounderlyingbytesource_type(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
    * @returns {number}
    */
    get autoAllocateChunkSize() {
        const ret = wasm.intounderlyingbytesource_autoAllocateChunkSize(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
    * @param {any} controller
    */
    start(controller) {
        wasm.intounderlyingbytesource_start(this.__wbg_ptr, addHeapObject(controller));
    }
    /**
    * @param {any} controller
    * @returns {Promise<any>}
    */
    pull(controller) {
        const ret = wasm.intounderlyingbytesource_pull(this.__wbg_ptr, addHeapObject(controller));
        return takeObject(ret);
    }
    /**
    */
    cancel() {
        const ptr = this.__destroy_into_raw();
        wasm.intounderlyingbytesource_cancel(ptr);
    }
}
/**
*/
export class IntoUnderlyingSink {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_intounderlyingsink_free(ptr);
    }
    /**
    * @param {any} chunk
    * @returns {Promise<any>}
    */
    write(chunk) {
        const ret = wasm.intounderlyingsink_write(this.__wbg_ptr, addHeapObject(chunk));
        return takeObject(ret);
    }
    /**
    * @returns {Promise<any>}
    */
    close() {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.intounderlyingsink_close(ptr);
        return takeObject(ret);
    }
    /**
    * @param {any} reason
    * @returns {Promise<any>}
    */
    abort(reason) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.intounderlyingsink_abort(ptr, addHeapObject(reason));
        return takeObject(ret);
    }
}
/**
*/
export class IntoUnderlyingSource {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_intounderlyingsource_free(ptr);
    }
    /**
    * @param {any} controller
    * @returns {Promise<any>}
    */
    pull(controller) {
        const ret = wasm.intounderlyingsource_pull(this.__wbg_ptr, addHeapObject(controller));
        return takeObject(ret);
    }
    /**
    */
    cancel() {
        const ptr = this.__destroy_into_raw();
        wasm.intounderlyingsource_cancel(ptr);
    }
}
/**
* Raw options for [`pipeTo()`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/pipeTo).
*/
export class PipeOptions {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_pipeoptions_free(ptr);
    }
    /**
    * @returns {boolean}
    */
    get preventClose() {
        const ret = wasm.pipeoptions_preventClose(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
    * @returns {boolean}
    */
    get preventCancel() {
        const ret = wasm.pipeoptions_preventCancel(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
    * @returns {boolean}
    */
    get preventAbort() {
        const ret = wasm.pipeoptions_preventAbort(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
    * @returns {AbortSignal | undefined}
    */
    get signal() {
        const ret = wasm.pipeoptions_signal(this.__wbg_ptr);
        return takeObject(ret);
    }
}
/**
*/
export class QueuingStrategy {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_queuingstrategy_free(ptr);
    }
    /**
    * @returns {number}
    */
    get highWaterMark() {
        const ret = wasm.queuingstrategy_highWaterMark(this.__wbg_ptr);
        return ret;
    }
}
/**
* Raw options for [`getReader()`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/getReader).
*/
export class ReadableStreamGetReaderOptions {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_readablestreamgetreaderoptions_free(ptr);
    }
    /**
    * @returns {any}
    */
    get mode() {
        const ret = wasm.readablestreamgetreaderoptions_mode(this.__wbg_ptr);
        return takeObject(ret);
    }
}
/**
*/
export class TombWasm {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(TombWasm.prototype);
        obj.__wbg_ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_tombwasm_free(ptr);
    }
    /**
    * Create a new TombWasm instance
    * # Arguments
    *
    * * `web_signing_key` - The CryptoKeyPair to use for signing requests
    * * `account_id` - The id of the account to use
    * * `api_endpoint` - The API endpoint to use
    *
    * # Returns
    *
    * A new TombWasm instance
    *
    * Don't call it from multiple threads in parallel!
    * @param {any} web_signing_key
    * @param {string} account_id
    * @param {string} api_endpoint
    */
    constructor(web_signing_key, account_id, api_endpoint) {
        const ptr0 = passStringToWasm0(account_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(api_endpoint, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.tombwasm_new(addHeapObject(web_signing_key), ptr0, len0, ptr1, len1);
        return TombWasm.__wrap(ret);
    }
    /**
    * Get the total consume storage space for the current account in bytes
    * @returns {Promise<bigint>}
    */
    getUsage() {
        const ret = wasm.tombwasm_getUsage(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
    * Get the current usage limit for the current account in bytes
    * @returns {Promise<bigint>}
    */
    getUsageLimit() {
        const ret = wasm.tombwasm_getUsageLimit(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
    * List the buckets for the current account
    * @returns {Promise<Array<any>>}
    */
    listBuckets() {
        const ret = wasm.tombwasm_listBuckets(this.__wbg_ptr);
        return takeObject(ret);
    }
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
    listBucketSnapshots(bucket_id) {
        const ptr0 = passStringToWasm0(bucket_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.tombwasm_listBucketSnapshots(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
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
    listBucketKeys(bucket_id) {
        const ptr0 = passStringToWasm0(bucket_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.tombwasm_listBucketKeys(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
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
    createBucket(name, storage_class, bucket_type, initial_key) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(storage_class, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(bucket_type, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.tombwasm_createBucket(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2, addHeapObject(initial_key));
        return takeObject(ret);
    }
    /**
    * Create a bucket key for a bucket
    * # Arguments
    * * `bucket_id` - The id of the bucket to create a key for
    * # Returns
    * The WasmBucketKey that was created
    * @param {string} bucket_id
    * @returns {Promise<WasmBucketKey>}
    */
    createBucketKey(bucket_id) {
        const ptr0 = passStringToWasm0(bucket_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.tombwasm_createBucketKey(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
    /**
    * Delete a bucket
    * # Arguments
    * * `bucket_id` - The id of the bucket to delete
    * # Returns the id of the bucket that was deleted
    * @param {string} bucket_id
    * @returns {Promise<void>}
    */
    deleteBucket(bucket_id) {
        const ptr0 = passStringToWasm0(bucket_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.tombwasm_deleteBucket(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
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
    mount(bucket_id, key) {
        const ptr0 = passStringToWasm0(bucket_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.tombwasm_mount(this.__wbg_ptr, ptr0, len0, addHeapObject(key));
        return takeObject(ret);
    }
}
/**
*/
export class WasmBucket {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmBucket.prototype);
        obj.__wbg_ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmbucket_free(ptr);
    }
    /**
    * @returns {string}
    */
    bucketType() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbucket_bucketType(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
    * @returns {string}
    */
    id() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbucket_id(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
    * @returns {string}
    */
    name() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbucket_name(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
    * @returns {string}
    */
    storageClass() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbucket_storageClass(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
/**
*/
export class WasmBucketKey {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmBucketKey.prototype);
        obj.__wbg_ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmbucketkey_free(ptr);
    }
    /**
    * @returns {boolean}
    */
    approved() {
        const ret = wasm.wasmbucketkey_approved(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
    * @returns {string}
    */
    bucketId() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbucketkey_bucketId(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
    * @returns {string}
    */
    id() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbucketkey_id(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
    * @returns {string}
    */
    pem() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbucketkey_pem(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
/**
* Mount point for a Bucket in WASM
*
* Enables to call Fs methods on a Bucket, pulling metadata from a remote
*/
export class WasmMount {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmMount.prototype);
        obj.__wbg_ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmmount_free(ptr);
    }
    /**
    * Returns whether or not the bucket is dirty (this will be true when a file or directory has
    * been changed).
    * @returns {boolean}
    */
    dirty() {
        const ret = wasm.wasmmount_dirty(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
    * Returns whether or not the bucket is locked
    * @returns {boolean}
    */
    locked() {
        const ret = wasm.wasmmount_locked(this.__wbg_ptr);
        return ret !== 0;
    }
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
    ls(path_segments) {
        const ret = wasm.wasmmount_ls(this.__wbg_ptr, addHeapObject(path_segments));
        return takeObject(ret);
    }
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
    mkdir(path_segments) {
        const ret = wasm.wasmmount_mkdir(this.__wbg_ptr, addHeapObject(path_segments));
        return takeObject(ret);
    }
    /**
    * Add a file
    * # Arguments
    * * `path_segments` - The path to add to (as an Array)
    * * `content_buffer` - The content to add (as an ArrayBuffer)
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
    add(path_segments, content_buffer) {
        const ret = wasm.wasmmount_add(this.__wbg_ptr, addHeapObject(path_segments), addHeapObject(content_buffer));
        return takeObject(ret);
    }
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
    * @returns {Promise<ArrayBuffer>}
    */
    readBytes(path_segments, _version) {
        var ptr0 = isLikeNone(_version) ? 0 : passStringToWasm0(_version, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmmount_readBytes(this.__wbg_ptr, addHeapObject(path_segments), ptr0, len0);
        return takeObject(ret);
    }
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
    mv(from_path_segments, to_path_segments) {
        const ret = wasm.wasmmount_mv(this.__wbg_ptr, addHeapObject(from_path_segments), addHeapObject(to_path_segments));
        return takeObject(ret);
    }
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
    rm(path_segments) {
        const ret = wasm.wasmmount_rm(this.__wbg_ptr, addHeapObject(path_segments));
        return takeObject(ret);
    }
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
    shareWith(bucket_key_id) {
        const ptr0 = passStringToWasm0(bucket_key_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmmount_shareWith(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
    /**
    * Snapshot a mounted bucket
    * # Returns
    * A Promise<void> in js speak
    * # Errors
    * * "missing metadata" - If the metadata is missing
    * * "could not snapshot" - If the snapshot fails
    * @returns {Promise<void>}
    */
    snapshot() {
        const ret = wasm.wasmmount_snapshot(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
    * Restore a mounted bucket
    * # Arguments
    * * `wasm_snapshot` - The snapshot to restore from
    * # Returns
    * A Promise<void> in js speak. Should update the mount to the version of the snapshot
    * @param {WasmSnapshot} wasm_snapshot
    * @returns {Promise<void>}
    */
    restore(wasm_snapshot) {
        _assertClass(wasm_snapshot, WasmSnapshot);
        var ptr0 = wasm_snapshot.__destroy_into_raw();
        const ret = wasm.wasmmount_restore(this.__wbg_ptr, ptr0);
        return takeObject(ret);
    }
}
/**
*/
export class WasmSnapshot {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmSnapshot.prototype);
        obj.__wbg_ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmsnapshot_free(ptr);
    }
    /**
    * @returns {string}
    */
    id() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmsnapshot_id(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
    * @returns {string}
    */
    bucketId() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmsnapshot_bucketId(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
    * @returns {string}
    */
    metadataId() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmsnapshot_metadataId(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
    * @returns {bigint}
    */
    dataSize() {
        const ret = wasm.wasmsnapshot_dataSize(this.__wbg_ptr);
        return ret;
    }
}

export function __wbindgen_object_drop_ref(arg0) {
    takeObject(arg0);
};

export function __wbindgen_string_new(arg0, arg1) {
    const ret = getStringFromWasm0(arg0, arg1);
    return addHeapObject(ret);
};

export function __wbindgen_object_clone_ref(arg0) {
    const ret = getObject(arg0);
    return addHeapObject(ret);
};

export function __wbindgen_bigint_from_u64(arg0) {
    const ret = BigInt.asUintN(64, arg0);
    return addHeapObject(ret);
};

export function __wbg_wasmbucket_new(arg0) {
    const ret = WasmBucket.__wrap(arg0);
    return addHeapObject(ret);
};

export function __wbg_wasmbucketkey_new(arg0) {
    const ret = WasmBucketKey.__wrap(arg0);
    return addHeapObject(ret);
};

export function __wbg_wasmmount_new(arg0) {
    const ret = WasmMount.__wrap(arg0);
    return addHeapObject(ret);
};

export function __wbg_wasmsnapshot_new(arg0) {
    const ret = WasmSnapshot.__wrap(arg0);
    return addHeapObject(ret);
};

export function __wbindgen_number_new(arg0) {
    const ret = arg0;
    return addHeapObject(ret);
};

export function __wbindgen_cb_drop(arg0) {
    const obj = takeObject(arg0).original;
    if (obj.cnt-- == 1) {
        obj.a = 0;
        return true;
    }
    const ret = false;
    return ret;
};

export function __wbindgen_string_get(arg0, arg1) {
    const obj = getObject(arg1);
    const ret = typeof(obj) === 'string' ? obj : undefined;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_new_abda76e883ba8a5f() {
    const ret = new Error();
    return addHeapObject(ret);
};

export function __wbg_stack_658279fe44541cf6(arg0, arg1) {
    const ret = getObject(arg1).stack;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_error_f851667af71bcfc6(arg0, arg1) {
    let deferred0_0;
    let deferred0_1;
    try {
        deferred0_0 = arg0;
        deferred0_1 = arg1;
        console.error(getStringFromWasm0(arg0, arg1));
    } finally {
        wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
    }
};

export function __wbindgen_is_string(arg0) {
    const ret = typeof(getObject(arg0)) === 'string';
    return ret;
};

export function __wbg_log_1f7f93998ab961f7(arg0, arg1) {
    var v0 = getArrayJsValueFromWasm0(arg0, arg1).slice();
    wasm.__wbindgen_free(arg0, arg1 * 4);
    console.log(...v0);
};

export function __wbg_now_2e07eedfb4ac9dbe() {
    const ret = Date.now();
    return ret;
};

export function __wbg_now_e2bd027927d3eced() {
    const ret = performance.now();
    return ret;
};

export function __wbg_fetch_381efb5e862610fa(arg0) {
    const ret = fetch(getObject(arg0));
    return addHeapObject(ret);
};

export function __wbg_respond_8fadc5f5c9d95422(arg0, arg1) {
    getObject(arg0).respond(arg1 >>> 0);
};

export function __wbg_byobRequest_08c18cee35def1f4(arg0) {
    const ret = getObject(arg0).byobRequest;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_view_231340b0dd8a2484(arg0) {
    const ret = getObject(arg0).view;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_close_e9110ca16e2567db(arg0) {
    getObject(arg0).close();
};

export function __wbg_enqueue_d71a1a518e21f5c3(arg0, arg1) {
    getObject(arg0).enqueue(getObject(arg1));
};

export function __wbg_byteLength_5299848ed3264181(arg0) {
    const ret = getObject(arg0).byteLength;
    return ret;
};

export function __wbg_close_da7e6fb9d9851e5a(arg0) {
    getObject(arg0).close();
};

export function __wbg_buffer_4e79326814bdd393(arg0) {
    const ret = getObject(arg0).buffer;
    return addHeapObject(ret);
};

export function __wbg_byteOffset_b69b0a07afccce19(arg0) {
    const ret = getObject(arg0).byteOffset;
    return ret;
};

export function __wbg_getReader_8ecba87d8003e950() { return handleError(function (arg0) {
    const ret = getObject(arg0).getReader();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_cancel_7f202496da02cd45(arg0) {
    const ret = getObject(arg0).cancel();
    return addHeapObject(ret);
};

export function __wbg_releaseLock_9ae075576f54bf0b() { return handleError(function (arg0) {
    getObject(arg0).releaseLock();
}, arguments) };

export function __wbg_read_88c96573fc8b3b01(arg0) {
    const ret = getObject(arg0).read();
    return addHeapObject(ret);
};

export function __wbg_done_76252d32deca186b(arg0) {
    const ret = getObject(arg0).done;
    return ret;
};

export function __wbg_value_ff3741eb46856618(arg0) {
    const ret = getObject(arg0).value;
    return addHeapObject(ret);
};

export function __wbg_instanceof_Window_9029196b662bc42a(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Window;
    } catch {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_crypto_6eb1167a1eff9af9() { return handleError(function (arg0) {
    const ret = getObject(arg0).crypto;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_fetch_8eaf01857a5bb21f(arg0, arg1) {
    const ret = getObject(arg0).fetch(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_deriveBits_64079d6036b39642() { return handleError(function (arg0, arg1, arg2, arg3) {
    const ret = getObject(arg0).deriveBits(getObject(arg1), getObject(arg2), arg3 >>> 0);
    return addHeapObject(ret);
}, arguments) };

export function __wbg_digest_8a7808f5a00c220e() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    const ret = getObject(arg0).digest(getStringFromWasm0(arg1, arg2), getArrayU8FromWasm0(arg3, arg4));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_exportKey_013c7fb7239a8b95() { return handleError(function (arg0, arg1, arg2, arg3) {
    const ret = getObject(arg0).exportKey(getStringFromWasm0(arg1, arg2), getObject(arg3));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_generateKey_2c5397375d30d16d() { return handleError(function (arg0, arg1, arg2, arg3) {
    const ret = getObject(arg0).generateKey(getObject(arg1), arg2 !== 0, getObject(arg3));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_importKey_a7e191d8d99dbbcf() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    const ret = getObject(arg0).importKey(getStringFromWasm0(arg1, arg2), getObject(arg3), getObject(arg4), arg5 !== 0, getObject(arg6));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_importKey_aac647d3e6a4dfa4() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
    const ret = getObject(arg0).importKey(getStringFromWasm0(arg1, arg2), getObject(arg3), getStringFromWasm0(arg4, arg5), arg6 !== 0, getObject(arg7));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_unwrapKey_ffb67e5fbe4a41ba() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
    const ret = getObject(arg0).unwrapKey(getStringFromWasm0(arg1, arg2), getArrayU8FromWasm0(arg3, arg4), getObject(arg5), getStringFromWasm0(arg6, arg7), getStringFromWasm0(arg8, arg9), arg10 !== 0, getObject(arg11));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_wrapKey_ddc03036fd70bfec() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    const ret = getObject(arg0).wrapKey(getStringFromWasm0(arg1, arg2), getObject(arg3), getObject(arg4), getStringFromWasm0(arg5, arg6));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_instanceof_Response_fc4327dbfcdf5ced(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Response;
    } catch {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_url_8503de97f69da463(arg0, arg1) {
    const ret = getObject(arg1).url;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_status_ac85a3142a84caa2(arg0) {
    const ret = getObject(arg0).status;
    return ret;
};

export function __wbg_headers_b70de86b8e989bc0(arg0) {
    const ret = getObject(arg0).headers;
    return addHeapObject(ret);
};

export function __wbg_body_b86f372950de5b7d(arg0) {
    const ret = getObject(arg0).body;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_arrayBuffer_288fb3538806e85c() { return handleError(function (arg0) {
    const ret = getObject(arg0).arrayBuffer();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_signal_4bd18fb489af2d4c(arg0) {
    const ret = getObject(arg0).signal;
    return addHeapObject(ret);
};

export function __wbg_new_55c9955722952374() { return handleError(function () {
    const ret = new AbortController();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_abort_654b796176d117aa(arg0) {
    getObject(arg0).abort();
};

export function __wbg_newwithu8arraysequenceandoptions_854056d2c35b489c() { return handleError(function (arg0, arg1) {
    const ret = new Blob(getObject(arg0), getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_subtle_be6a0d5964ce84c8(arg0) {
    const ret = getObject(arg0).subtle;
    return addHeapObject(ret);
};

export function __wbg_getRandomValues_38b282e424c9b486() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).getRandomValues(getArrayU8FromWasm0(arg1, arg2));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_new_3352afcefc74e6b9() { return handleError(function () {
    const ret = new FormData();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_append_ae2160b26eaa2425() { return handleError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).append(getStringFromWasm0(arg1, arg2), getObject(arg3));
}, arguments) };

export function __wbg_append_161827fd6f926e64() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).append(getStringFromWasm0(arg1, arg2), getObject(arg3), getStringFromWasm0(arg4, arg5));
}, arguments) };

export function __wbg_newwithstrandinit_cad5cd6038c7ff5d() { return handleError(function (arg0, arg1, arg2) {
    const ret = new Request(getStringFromWasm0(arg0, arg1), getObject(arg2));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_new_1eead62f64ca15ce() { return handleError(function () {
    const ret = new Headers();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_append_fda9e3432e3e88da() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).append(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
}, arguments) };

export function __wbg_crypto_c48a774b022d20ac(arg0) {
    const ret = getObject(arg0).crypto;
    return addHeapObject(ret);
};

export function __wbindgen_is_object(arg0) {
    const val = getObject(arg0);
    const ret = typeof(val) === 'object' && val !== null;
    return ret;
};

export function __wbg_process_298734cf255a885d(arg0) {
    const ret = getObject(arg0).process;
    return addHeapObject(ret);
};

export function __wbg_versions_e2e78e134e3e5d01(arg0) {
    const ret = getObject(arg0).versions;
    return addHeapObject(ret);
};

export function __wbg_node_1cd7a5d853dbea79(arg0) {
    const ret = getObject(arg0).node;
    return addHeapObject(ret);
};

export function __wbg_msCrypto_bcb970640f50a1e8(arg0) {
    const ret = getObject(arg0).msCrypto;
    return addHeapObject(ret);
};

export function __wbg_require_8f08ceecec0f4fee() { return handleError(function () {
    const ret = module.require;
    return addHeapObject(ret);
}, arguments) };

export function __wbindgen_is_function(arg0) {
    const ret = typeof(getObject(arg0)) === 'function';
    return ret;
};

export function __wbg_randomFillSync_dc1e9a60c158336d() { return handleError(function (arg0, arg1) {
    getObject(arg0).randomFillSync(takeObject(arg1));
}, arguments) };

export function __wbg_getRandomValues_37fa2ca9e4e07fab() { return handleError(function (arg0, arg1) {
    getObject(arg0).getRandomValues(getObject(arg1));
}, arguments) };

export function __wbg_get_44be0491f933a435(arg0, arg1) {
    const ret = getObject(arg0)[arg1 >>> 0];
    return addHeapObject(ret);
};

export function __wbg_length_fff51ee6522a1a18(arg0) {
    const ret = getObject(arg0).length;
    return ret;
};

export function __wbg_new_898a68150f225f2e() {
    const ret = new Array();
    return addHeapObject(ret);
};

export function __wbg_newnoargs_581967eacc0e2604(arg0, arg1) {
    const ret = new Function(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
};

export function __wbg_next_526fc47e980da008(arg0) {
    const ret = getObject(arg0).next;
    return addHeapObject(ret);
};

export function __wbg_next_ddb3312ca1c4e32a() { return handleError(function (arg0) {
    const ret = getObject(arg0).next();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_done_5c1f01fb660d73b5(arg0) {
    const ret = getObject(arg0).done;
    return ret;
};

export function __wbg_value_1695675138684bd5(arg0) {
    const ret = getObject(arg0).value;
    return addHeapObject(ret);
};

export function __wbg_iterator_97f0c81209c6c35a() {
    const ret = Symbol.iterator;
    return addHeapObject(ret);
};

export function __wbg_get_97b561fb56f034b5() { return handleError(function (arg0, arg1) {
    const ret = Reflect.get(getObject(arg0), getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_call_cb65541d95d71282() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).call(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_new_b51585de1b234aff() {
    const ret = new Object();
    return addHeapObject(ret);
};

export function __wbg_self_1ff1d729e9aae938() { return handleError(function () {
    const ret = self.self;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_window_5f4faef6c12b79ec() { return handleError(function () {
    const ret = window.window;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_globalThis_1d39714405582d3c() { return handleError(function () {
    const ret = globalThis.globalThis;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_global_651f05c6a0944d1c() { return handleError(function () {
    const ret = global.global;
    return addHeapObject(ret);
}, arguments) };

export function __wbindgen_is_undefined(arg0) {
    const ret = getObject(arg0) === undefined;
    return ret;
};

export function __wbg_push_ca1c26067ef907ac(arg0, arg1) {
    const ret = getObject(arg0).push(getObject(arg1));
    return ret;
};

export function __wbg_instanceof_ArrayBuffer_39ac22089b74fddb(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof ArrayBuffer;
    } catch {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_byteLength_0488a7a303dccf40(arg0) {
    const ret = getObject(arg0).byteLength;
    return ret;
};

export function __wbg_new_d258248ed531ff54(arg0, arg1) {
    const ret = new Error(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
};

export function __wbg_call_01734de55d61e11d() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_getTime_5e2054f832d82ec9(arg0) {
    const ret = getObject(arg0).getTime();
    return ret;
};

export function __wbg_new0_c0be7df4b6bd481f() {
    const ret = new Date();
    return addHeapObject(ret);
};

export function __wbg_instanceof_Object_3daa8298c86298be(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Object;
    } catch {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_new_43f1b47c28813cbd(arg0, arg1) {
    try {
        var state0 = {a: arg0, b: arg1};
        var cb0 = (arg0, arg1) => {
            const a = state0.a;
            state0.a = 0;
            try {
                return __wbg_adapter_250(a, state0.b, arg0, arg1);
            } finally {
                state0.a = a;
            }
        };
        const ret = new Promise(cb0);
        return addHeapObject(ret);
    } finally {
        state0.a = state0.b = 0;
    }
};

export function __wbg_resolve_53698b95aaf7fcf8(arg0) {
    const ret = Promise.resolve(getObject(arg0));
    return addHeapObject(ret);
};

export function __wbg_catch_64e0c7dcea0da34e(arg0, arg1) {
    const ret = getObject(arg0).catch(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_then_f7e06ee3c11698eb(arg0, arg1) {
    const ret = getObject(arg0).then(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_then_b2267541e2a73865(arg0, arg1, arg2) {
    const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
};

export function __wbg_buffer_085ec1f694018c4f(arg0) {
    const ret = getObject(arg0).buffer;
    return addHeapObject(ret);
};

export function __wbg_newwithbyteoffsetandlength_6da8e527659b86aa(arg0, arg1, arg2) {
    const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_new_8125e318e6245eed(arg0) {
    const ret = new Uint8Array(getObject(arg0));
    return addHeapObject(ret);
};

export function __wbg_set_5cf90238115182c3(arg0, arg1, arg2) {
    getObject(arg0).set(getObject(arg1), arg2 >>> 0);
};

export function __wbg_length_72e2208bbc0efc61(arg0) {
    const ret = getObject(arg0).length;
    return ret;
};

export function __wbg_newwithlength_e5d69174d6984cd7(arg0) {
    const ret = new Uint8Array(arg0 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_buffer_f5b7059c439f330d(arg0) {
    const ret = getObject(arg0).buffer;
    return addHeapObject(ret);
};

export function __wbg_subarray_13db269f57aa838d(arg0, arg1, arg2) {
    const ret = getObject(arg0).subarray(arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_stringify_e25465938f3f611f() { return handleError(function (arg0) {
    const ret = JSON.stringify(getObject(arg0));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_has_c5fcd020291e56b8() { return handleError(function (arg0, arg1) {
    const ret = Reflect.has(getObject(arg0), getObject(arg1));
    return ret;
}, arguments) };

export function __wbg_set_092e06b0f9d71865() { return handleError(function (arg0, arg1, arg2) {
    const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
    return ret;
}, arguments) };

export function __wbindgen_debug_string(arg0, arg1) {
    const ret = debugString(getObject(arg1));
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbindgen_throw(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

export function __wbindgen_rethrow(arg0) {
    throw takeObject(arg0);
};

export function __wbindgen_memory() {
    const ret = wasm.memory;
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper2965(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1217, __wbg_adapter_32);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper3019(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1229, __wbg_adapter_35);
    return addHeapObject(ret);
};

