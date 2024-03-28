import * as wasm from "./banyanfs_bg.wasm";
import { __wbg_set_wasm } from "./banyanfs_bg.js";
__wbg_set_wasm(wasm);
export * from "./banyanfs_bg.js";

wasm.__wbindgen_start();
