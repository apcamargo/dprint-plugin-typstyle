pub mod configuration;
mod format_text;

pub use format_text::format_text;

#[cfg(all(feature = "wasm", target_arch = "wasm32", target_os = "unknown"))]
mod wasm_plugin;
#[cfg(all(feature = "wasm", target_arch = "wasm32", target_os = "unknown"))]
pub use wasm_plugin::*;
