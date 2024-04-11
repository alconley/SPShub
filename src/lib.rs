// Activate compiler warnings for all Clippy lints and idiomatic Rust 2018 practices.
#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::SPSHubApp; 
