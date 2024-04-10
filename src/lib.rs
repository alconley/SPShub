// Activate compiler warnings for all Clippy lints and idiomatic Rust 2018 practices.
#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::TemplateApp; 

mod sps_runtime_estimator;
mod cebra_efficiency;

#[cfg(not(target_arch = "wasm32"))]
mod sps_plot;

#[cfg(not(target_arch = "wasm32"))]
mod plotter;
// The following modules are conditionally compiled only for non-WASM targets, 
// indicating they contain functionality specific to native platforms.
#[cfg(not(target_arch = "wasm32"))]
mod sps_cebra_eventbuilder;

#[cfg(not(target_arch = "wasm32"))]
mod cebra_eventbuilder;

