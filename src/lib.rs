// Activate compiler warnings for all Clippy lints and idiomatic Rust 2018 practices.
#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::TemplateApp; 

// The following modules are conditionally compiled only for non-WASM targets, 
// indicating they contain functionality specific to native platforms.
#[cfg(not(target_arch = "wasm32"))]
mod sps_cebra_eventbuilder;

#[cfg(not(target_arch = "wasm32"))]
mod sps_eventbuilder;