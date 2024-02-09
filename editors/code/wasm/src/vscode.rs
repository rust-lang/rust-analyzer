pub mod window {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = vscode_window)]
        pub fn showWarningMessage(message: &str, button: &str);
    }
}

pub mod extensions {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = vscode_extensions)]
        pub fn getExtension(s: &str) -> JsValue;
    }
}
