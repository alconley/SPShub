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


