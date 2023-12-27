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
    wasm.wasm_bindgen__convert__closures__invoke1_mut__hbbdb9a32b3f7fba4(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_35(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h7951dbd10cda50b0(arg0, arg1, addHeapObject(arg2));
}

/**
*/
export function register_log() {
    wasm.register_log();
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
    return instance.ptr;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_exn_store(addHeapObject(e));
    }
}
function __wbg_adapter_267(arg0, arg1, arg2, arg3) {
    wasm.wasm_bindgen__convert__closures__invoke2_mut__h3626b1ce1ccc8f20(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
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
* Wrapper around a Client
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
    * * `user_id` - The id of the account to use
    * * `core_endpoint` - The API endpoint to use for core
    *
    * # Returns
    *
    * A new TombWasm instance
    *
    * Don't call it from multiple threads in parallel!
    * @param {string} signing_key_pem
    * @param {string} user_id
    * @param {string} core_endpoint
    */
    constructor(signing_key_pem, user_id, core_endpoint) {
        const ptr0 = passStringToWasm0(signing_key_pem, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(user_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(core_endpoint, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.tombwasm_new(ptr0, len0, ptr1, len1, ptr2, len2);
        return takeObject(ret);
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
    * * `private_pem` - The private encryption key to use for the bucket
    * * `public_pem` - The public encryption key to use for the bucket
    * # Returns
    * The bucket's metadata as a WasmBucket
    * ```json
    * {
    * "name": "string"
    * "storage_class": "string",
    * "bucket_type": "string",
    * "private_pem": "string",
    * "public_pem": "string",
    * }
    * ```
    * @param {string} name
    * @param {string} storage_class
    * @param {string} bucket_type
    * @param {string} private_pem
    * @param {string} public_pem
    * @returns {Promise<WasmBucketMount>}
    */
    createBucketAndMount(name, storage_class, bucket_type, private_pem, public_pem) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(storage_class, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(bucket_type, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ptr3 = passStringToWasm0(private_pem, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len3 = WASM_VECTOR_LEN;
        const ptr4 = passStringToWasm0(public_pem, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len4 = WASM_VECTOR_LEN;
        const ret = wasm.tombwasm_createBucketAndMount(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3, ptr4, len4);
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
    * Rename a bucket
    * # Arguments
    * * `bucket_id` - The id of the bucket to rename
    * * `name` - the new name to give to the bucket
    * # Returns Promise<void> in js speak
    * @param {string} bucket_id
    * @param {string} name
    * @returns {Promise<void>}
    */
    renameBucket(bucket_id, name) {
        const ptr0 = passStringToWasm0(bucket_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.tombwasm_renameBucket(this.__wbg_ptr, ptr0, len0, ptr1, len1);
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
    * Approve the device key and end Registration waiting
    * # Arguments
    * * pem - The public PEM of the key being approved
    * # Returns
    * The result of ending the registration wait
    * @param {string} pem
    * @returns {Promise<void>}
    */
    approveDeviceApiKey(pem) {
        const ptr0 = passStringToWasm0(pem, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.tombwasm_approveDeviceApiKey(this.__wbg_ptr, ptr0, len0);
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
    * @param {string} encryption_key_pem
    * @returns {Promise<WasmMount>}
    */
    mount(bucket_id, encryption_key_pem) {
        const ptr0 = passStringToWasm0(bucket_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(encryption_key_pem, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.tombwasm_mount(this.__wbg_ptr, ptr0, len0, ptr1, len1);
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
*/
export class WasmBucketMetadata {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmBucketMetadata.prototype);
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
        wasm.__wbg_wasmbucketmetadata_free(ptr);
    }
    /**
    * @returns {string}
    */
    get id() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbucketmetadata_id(retptr, this.__wbg_ptr);
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
    get bucketId() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbucketmetadata_bucket_id(retptr, this.__wbg_ptr);
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
    get snapshotId() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbucketmetadata_snapshot_id(retptr, this.__wbg_ptr);
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
export class WasmBucketMount {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmBucketMount.prototype);
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
        wasm.__wbg_wasmbucketmount_free(ptr);
    }
    /**
    * @param {WasmBucket} bucket
    * @param {WasmMount} mount
    * @returns {WasmBucketMount}
    */
    static new(bucket, mount) {
        _assertClass(bucket, WasmBucket);
        var ptr0 = bucket.__destroy_into_raw();
        _assertClass(mount, WasmMount);
        var ptr1 = mount.__destroy_into_raw();
        const ret = wasm.wasmbucketmount_new(ptr0, ptr1);
        return WasmBucketMount.__wrap(ret);
    }
    /**
    * @returns {WasmBucket}
    */
    get bucket() {
        const ret = wasm.wasmbucketmount_bucket(this.__wbg_ptr);
        return WasmBucket.__wrap(ret);
    }
    /**
    * @returns {WasmMount}
    */
    get mount() {
        const ret = wasm.wasmbucketmount_mount(this.__wbg_ptr);
        return WasmMount.__wrap(ret);
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
    * Returns the Bucket behind the mount
    * @returns {WasmBucket}
    */
    bucket() {
        const ret = wasm.wasmmount_bucket(this.__wbg_ptr);
        return WasmBucket.__wrap(ret);
    }
    /**
    * Returns the Metadata for the bucket
    * @returns {WasmBucketMetadata}
    */
    metadata() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmmount_metadata(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var r2 = getInt32Memory0()[retptr / 4 + 2];
            if (r2) {
                throw takeObject(r1);
            }
            return WasmBucketMetadata.__wrap(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
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
    write(path_segments, content_buffer) {
        const ret = wasm.wasmmount_write(this.__wbg_ptr, addHeapObject(path_segments), addHeapObject(content_buffer));
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
    * @param {string | undefined} [_version]
    * @returns {Promise<Uint8Array>}
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
    * Share a file snapshot
    * @param {Array<any>} path_segments
    * @returns {Promise<string>}
    */
    shareFile(path_segments) {
        const ret = wasm.wasmmount_shareFile(this.__wbg_ptr, addHeapObject(path_segments));
        return takeObject(ret);
    }
    /**
    * Return boolean indiciating whether or not the currently mounted bucket is snapshotted
    * # Returns
    * A boolean
    * # Errors
    * * "missing metadata" - If the metadata is missing
    * @returns {boolean}
    */
    hasSnapshot() {
        const ret = wasm.wasmmount_hasSnapshot(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
    * Snapshot a mounted bucket
    * # Returns
    * A Promise<void> in js speak
    * # Errors
    * * "missing metadata" - If the metadata is missing
    * * "could not snapshot" - If the snapshot fails
    * @returns {Promise<string>}
    */
    snapshot() {
        const ret = wasm.wasmmount_snapshot(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
    * Rename the mounted bucket
    * # Arguments
    * * `name` - the new name for the bucket
    * # Returns
    * A Promise<void> in js speak. Should also update the internal state of the bucket
    * on a successful update
    * @param {string} name
    * @returns {Promise<void>}
    */
    rename(name) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmmount_rename(this.__wbg_ptr, ptr0, len0);
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
export class WasmSharedFile {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmSharedFile.prototype);
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
        wasm.__wbg_wasmsharedfile_free(ptr);
    }
    /**
    * @returns {string}
    */
    export_b64_url() {
        let deferred2_0;
        let deferred2_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmsharedfile_export_b64_url(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var r2 = getInt32Memory0()[retptr / 4 + 2];
            var r3 = getInt32Memory0()[retptr / 4 + 3];
            var ptr1 = r0;
            var len1 = r1;
            if (r3) {
                ptr1 = 0; len1 = 0;
                throw takeObject(r2);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
    * @param {string} b64_string
    * @returns {WasmSharedFile}
    */
    static import_b64_url(b64_string) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(b64_string, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmsharedfile_import_b64_url(retptr, ptr0, len0);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var r2 = getInt32Memory0()[retptr / 4 + 2];
            if (r2) {
                throw takeObject(r1);
            }
            return WasmSharedFile.__wrap(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * @returns {string | undefined}
    */
    mimeType() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmsharedfile_mimeType(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            let v1;
            if (r0 !== 0) {
                v1 = getStringFromWasm0(r0, r1).slice();
                wasm.__wbindgen_free(r0, r1 * 1, 1);
            }
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * @returns {string | undefined}
    */
    size() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmsharedfile_size(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            let v1;
            if (r0 !== 0) {
                v1 = getStringFromWasm0(r0, r1).slice();
                wasm.__wbindgen_free(r0, r1 * 1, 1);
            }
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * @returns {string}
    */
    fileName() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmsharedfile_fileName(retptr, this.__wbg_ptr);
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
    get bucketId() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmsnapshot_bucket_id(retptr, this.__wbg_ptr);
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
    get createdAt() {
        const ret = wasm.wasmsnapshot_created_at(this.__wbg_ptr);
        return ret;
    }
    /**
    * @returns {string}
    */
    get id() {
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
    get metadataId() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmsnapshot_metadata_id(retptr, this.__wbg_ptr);
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
    get size() {
        const ret = wasm.wasmsnapshot_size(this.__wbg_ptr);
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

export function __wbindgen_bigint_from_u64(arg0) {
    const ret = BigInt.asUintN(64, arg0);
    return addHeapObject(ret);
};

export function __wbg_tombwasm_new(arg0) {
    const ret = TombWasm.__wrap(arg0);
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

export function __wbg_wasmbucketmount_new(arg0) {
    const ret = WasmBucketMount.__wrap(arg0);
    return addHeapObject(ret);
};

export function __wbindgen_is_undefined(arg0) {
    const ret = getObject(arg0) === undefined;
    return ret;
};

export function __wbg_wasmsnapshot_new(arg0) {
    const ret = WasmSnapshot.__wrap(arg0);
    return addHeapObject(ret);
};

export function __wbg_wasmbucket_new(arg0) {
    const ret = WasmBucket.__wrap(arg0);
    return addHeapObject(ret);
};

export function __wbindgen_string_get(arg0, arg1) {
    const obj = getObject(arg1);
    const ret = typeof(obj) === 'string' ? obj : undefined;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
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

export function __wbindgen_number_new(arg0) {
    const ret = arg0;
    return addHeapObject(ret);
};

export function __wbg_mark_6045ef1772587264() { return handleError(function (arg0, arg1, arg2) {
    getObject(arg0).mark(getStringFromWasm0(arg1, arg2));
}, arguments) };

export function __wbg_mark_bad820680b8580c2() { return handleError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).mark(getStringFromWasm0(arg1, arg2), getObject(arg3));
}, arguments) };

export function __wbg_measure_1d846b814d43d7e1() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).measure(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4), getStringFromWasm0(arg5, arg6));
}, arguments) };

export function __wbg_measure_7ca0e5cfef892340() { return handleError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).measure(getStringFromWasm0(arg1, arg2), getObject(arg3));
}, arguments) };

export function __wbg_performance_72f95fe5952939b5() {
    const ret = globalThis.performance;
    return addHeapObject(ret);
};

export function __wbg_now_0343d9c3e0e8eedc() {
    const ret = Date.now();
    return ret;
};

export function __wbg_crypto_58f13aa23ffcb166(arg0) {
    const ret = getObject(arg0).crypto;
    return addHeapObject(ret);
};

export function __wbindgen_is_object(arg0) {
    const val = getObject(arg0);
    const ret = typeof(val) === 'object' && val !== null;
    return ret;
};

export function __wbg_process_5b786e71d465a513(arg0) {
    const ret = getObject(arg0).process;
    return addHeapObject(ret);
};

export function __wbg_versions_c2ab80650590b6a2(arg0) {
    const ret = getObject(arg0).versions;
    return addHeapObject(ret);
};

export function __wbg_node_523d7bd03ef69fba(arg0) {
    const ret = getObject(arg0).node;
    return addHeapObject(ret);
};

export function __wbindgen_is_string(arg0) {
    const ret = typeof(getObject(arg0)) === 'string';
    return ret;
};

export function __wbg_msCrypto_abcb1295e768d1f2(arg0) {
    const ret = getObject(arg0).msCrypto;
    return addHeapObject(ret);
};

export function __wbg_require_2784e593a4674877() { return handleError(function () {
    const ret = module.require;
    return addHeapObject(ret);
}, arguments) };

export function __wbindgen_is_function(arg0) {
    const ret = typeof(getObject(arg0)) === 'function';
    return ret;
};

export function __wbg_randomFillSync_a0d98aa11c81fe89() { return handleError(function (arg0, arg1) {
    getObject(arg0).randomFillSync(takeObject(arg1));
}, arguments) };

export function __wbg_getRandomValues_504510b5564925af() { return handleError(function (arg0, arg1) {
    getObject(arg0).getRandomValues(getObject(arg1));
}, arguments) };

export function __wbg_fetch_b5d6bebed1e6c2d2(arg0) {
    const ret = fetch(getObject(arg0));
    return addHeapObject(ret);
};

export function __wbindgen_object_clone_ref(arg0) {
    const ret = getObject(arg0);
    return addHeapObject(ret);
};

export function __wbg_respond_8fadc5f5c9d95422(arg0, arg1) {
    getObject(arg0).respond(arg1 >>> 0);
};

export function __wbg_getReader_8ecba87d8003e950() { return handleError(function (arg0) {
    const ret = getObject(arg0).getReader();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_releaseLock_9ae075576f54bf0b() { return handleError(function (arg0) {
    getObject(arg0).releaseLock();
}, arguments) };

export function __wbg_cancel_7f202496da02cd45(arg0) {
    const ret = getObject(arg0).cancel();
    return addHeapObject(ret);
};

export function __wbg_close_e9110ca16e2567db(arg0) {
    getObject(arg0).close();
};

export function __wbg_enqueue_d71a1a518e21f5c3(arg0, arg1) {
    getObject(arg0).enqueue(getObject(arg1));
};

export function __wbg_byobRequest_08c18cee35def1f4(arg0) {
    const ret = getObject(arg0).byobRequest;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_close_da7e6fb9d9851e5a(arg0) {
    getObject(arg0).close();
};

export function __wbg_view_231340b0dd8a2484(arg0) {
    const ret = getObject(arg0).view;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_buffer_4e79326814bdd393(arg0) {
    const ret = getObject(arg0).buffer;
    return addHeapObject(ret);
};

export function __wbg_byteOffset_b69b0a07afccce19(arg0) {
    const ret = getObject(arg0).byteOffset;
    return ret;
};

export function __wbg_byteLength_5299848ed3264181(arg0) {
    const ret = getObject(arg0).byteLength;
    return ret;
};

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

export function __wbg_queueMicrotask_e5949c35d772a669(arg0) {
    queueMicrotask(getObject(arg0));
};

export function __wbg_queueMicrotask_2be8b97a81fe4d00(arg0) {
    const ret = getObject(arg0).queueMicrotask;
    return addHeapObject(ret);
};

export function __wbg_fetch_701fcd2bde06379a(arg0, arg1) {
    const ret = getObject(arg0).fetch(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_debug_2ef5d777cf4811fa(arg0) {
    console.debug(getObject(arg0));
};

export function __wbg_debug_8f9a97dc395d342f(arg0, arg1, arg2, arg3) {
    console.debug(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
};

export function __wbg_error_f0a6627f4b23c19d(arg0) {
    console.error(getObject(arg0));
};

export function __wbg_error_94a25ece8eeb7bca(arg0, arg1, arg2, arg3) {
    console.error(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
};

export function __wbg_info_3ca7870690403fee(arg0) {
    console.info(getObject(arg0));
};

export function __wbg_info_1d035e3d63b89260(arg0, arg1, arg2, arg3) {
    console.info(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
};

export function __wbg_warn_4affe1093892a4ef(arg0) {
    console.warn(getObject(arg0));
};

export function __wbg_warn_fab4b297e5c436a0(arg0, arg1, arg2, arg3) {
    console.warn(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
};

export function __wbg_newwithstrandinit_29038da14d09e330() { return handleError(function (arg0, arg1, arg2) {
    const ret = new Request(getStringFromWasm0(arg0, arg1), getObject(arg2));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_signal_1ed842bebd6ae322(arg0) {
    const ret = getObject(arg0).signal;
    return addHeapObject(ret);
};

export function __wbg_new_e4960143e41697a4() { return handleError(function () {
    const ret = new AbortController();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_abort_8355f201f30300bb(arg0) {
    getObject(arg0).abort();
};

export function __wbg_newwithu8arraysequenceandoptions_f520ece5c28a5211() { return handleError(function (arg0, arg1) {
    const ret = new Blob(getObject(arg0), getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_new_8a3eb39f9c444a2e() { return handleError(function () {
    const ret = new FormData();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_append_ac6b793c0e0232cb() { return handleError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).append(getStringFromWasm0(arg1, arg2), getObject(arg3));
}, arguments) };

export function __wbg_append_7ad942045bdb5a5a() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).append(getStringFromWasm0(arg1, arg2), getObject(arg3), getStringFromWasm0(arg4, arg5));
}, arguments) };

export function __wbg_new_19676474aa414d62() { return handleError(function () {
    const ret = new Headers();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_append_feec4143bbf21904() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).append(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
}, arguments) };

export function __wbg_instanceof_Response_944e2745b5db71f5(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Response;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_url_1f609e63ff1a7983(arg0, arg1) {
    const ret = getObject(arg1).url;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_status_7841bb47be2a8f16(arg0) {
    const ret = getObject(arg0).status;
    return ret;
};

export function __wbg_headers_ea7ef583d1564b08(arg0) {
    const ret = getObject(arg0).headers;
    return addHeapObject(ret);
};

export function __wbg_body_013f6a6a04d0ff92(arg0) {
    const ret = getObject(arg0).body;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_arrayBuffer_e32d72b052ba31d7() { return handleError(function (arg0) {
    const ret = getObject(arg0).arrayBuffer();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_get_4a9aa5157afeb382(arg0, arg1) {
    const ret = getObject(arg0)[arg1 >>> 0];
    return addHeapObject(ret);
};

export function __wbg_length_cace2e0b3ddc0502(arg0) {
    const ret = getObject(arg0).length;
    return ret;
};

export function __wbg_new_08236689f0afb357() {
    const ret = new Array();
    return addHeapObject(ret);
};

export function __wbg_newnoargs_ccdcae30fd002262(arg0, arg1) {
    const ret = new Function(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
};

export function __wbg_next_15da6a3df9290720(arg0) {
    const ret = getObject(arg0).next;
    return addHeapObject(ret);
};

export function __wbg_next_1989a20442400aaa() { return handleError(function (arg0) {
    const ret = getObject(arg0).next();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_done_bc26bf4ada718266(arg0) {
    const ret = getObject(arg0).done;
    return ret;
};

export function __wbg_value_0570714ff7d75f35(arg0) {
    const ret = getObject(arg0).value;
    return addHeapObject(ret);
};

export function __wbg_iterator_7ee1a391d310f8e4() {
    const ret = Symbol.iterator;
    return addHeapObject(ret);
};

export function __wbg_get_2aff440840bb6202() { return handleError(function (arg0, arg1) {
    const ret = Reflect.get(getObject(arg0), getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_call_669127b9d730c650() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).call(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_new_c728d68b8b34487e() {
    const ret = new Object();
    return addHeapObject(ret);
};

export function __wbg_self_3fad056edded10bd() { return handleError(function () {
    const ret = self.self;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_window_a4f46c98a61d4089() { return handleError(function () {
    const ret = window.window;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_globalThis_17eff828815f7d84() { return handleError(function () {
    const ret = globalThis.globalThis;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_global_46f939f6541643c5() { return handleError(function () {
    const ret = global.global;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_join_3d5c93f25195511b(arg0, arg1, arg2) {
    const ret = getObject(arg0).join(getStringFromWasm0(arg1, arg2));
    return addHeapObject(ret);
};

export function __wbg_push_fd3233d09cf81821(arg0, arg1) {
    const ret = getObject(arg0).push(getObject(arg1));
    return ret;
};

export function __wbg_new_ab87fd305ed9004b(arg0, arg1) {
    const ret = new Error(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
};

export function __wbg_call_53fc3abd42e24ec8() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_getTime_ed6ee333b702f8fc(arg0) {
    const ret = getObject(arg0).getTime();
    return ret;
};

export function __wbg_new0_ad75dd38f92424e2() {
    const ret = new Date();
    return addHeapObject(ret);
};

export function __wbg_create_9b9e7caad35d0488(arg0) {
    const ret = Object.create(getObject(arg0));
    return addHeapObject(ret);
};

export function __wbg_new_feb65b865d980ae2(arg0, arg1) {
    try {
        var state0 = {a: arg0, b: arg1};
        var cb0 = (arg0, arg1) => {
            const a = state0.a;
            state0.a = 0;
            try {
                return __wbg_adapter_267(a, state0.b, arg0, arg1);
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

export function __wbg_resolve_a3252b2860f0a09e(arg0) {
    const ret = Promise.resolve(getObject(arg0));
    return addHeapObject(ret);
};

export function __wbg_catch_5571771e01b2aec9(arg0, arg1) {
    const ret = getObject(arg0).catch(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_then_89e1c559530b85cf(arg0, arg1) {
    const ret = getObject(arg0).then(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_then_1bbc9edafd859b06(arg0, arg1, arg2) {
    const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
};

export function __wbg_buffer_344d9b41efe96da7(arg0) {
    const ret = getObject(arg0).buffer;
    return addHeapObject(ret);
};

export function __wbg_newwithbyteoffsetandlength_2dc04d99088b15e3(arg0, arg1, arg2) {
    const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_new_d8a000788389a31e(arg0) {
    const ret = new Uint8Array(getObject(arg0));
    return addHeapObject(ret);
};

export function __wbg_set_dcfd613a3420f908(arg0, arg1, arg2) {
    getObject(arg0).set(getObject(arg1), arg2 >>> 0);
};

export function __wbg_length_a5587d6cd79ab197(arg0) {
    const ret = getObject(arg0).length;
    return ret;
};

export function __wbg_newwithlength_13b5319ab422dcf6(arg0) {
    const ret = new Uint8Array(arg0 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_subarray_6ca5cfa7fbb9abbe(arg0, arg1, arg2) {
    const ret = getObject(arg0).subarray(arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_stringify_4039297315a25b00() { return handleError(function (arg0) {
    const ret = JSON.stringify(getObject(arg0));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_has_cdf8b85f6e903c80() { return handleError(function (arg0, arg1) {
    const ret = Reflect.has(getObject(arg0), getObject(arg1));
    return ret;
}, arguments) };

export function __wbg_set_40f7786a25a9cc7e() { return handleError(function (arg0, arg1, arg2) {
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

export function __wbindgen_closure_wrapper3733(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1440, __wbg_adapter_32);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper3817(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1474, __wbg_adapter_35);
    return addHeapObject(ret);
};

