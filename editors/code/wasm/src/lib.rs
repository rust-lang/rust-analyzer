use wasm_bindgen::prelude::*;

pub mod vscode;

#[wasm_bindgen]
pub fn check_conflicting_extensions() {
    if vscode::extensions::getExtension("rust-lang.rust").is_truthy() {
        vscode::window::showWarningMessage(
            "You have both the rust-analyzer (rust-lang.rust-analyzer) and Rust (rust-lang.rust) \
                plugins enabled. These are known to conflict and cause various functions of \
                both plugins to not work correctly. You should disable one of them.",
            "Got it",
        );
    }

    if vscode::extensions::getExtension("panicbit.cargo").is_truthy() {
        vscode::window::showWarningMessage(
            "You have both the rust-analyzer (rust-lang.rust-analyzer) and Cargo (panicbit.cargo) plugins enabled, \
            you can disable it or set {'cargo.automaticCheck': false} in settings.json to avoid invoking cargo twice",
            "Got it",
        );
    }
}
