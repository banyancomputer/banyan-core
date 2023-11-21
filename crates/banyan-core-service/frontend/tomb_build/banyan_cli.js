import * as wasm from "./banyan_cli_bg.wasm";
import { __wbg_set_wasm } from "./banyan_cli_bg.js";
__wbg_set_wasm(wasm);
export * from "./banyan_cli_bg.js";
