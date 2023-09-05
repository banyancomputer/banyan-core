import * as wasm from "./tomb_wasm_bg.wasm";
import { __wbg_set_wasm } from "./tomb_wasm_bg.js";
__wbg_set_wasm(wasm);
export * from "./tomb_wasm_bg.js";
