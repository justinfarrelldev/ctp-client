//! # CTP Client Library
//!
//! A custom Call to Power client library for strategic civilization gameplay.
//! This library provides core functionality for game installation detection,
//! configuration management, and user interface components.
//!
//! ## Features
//!
//! - Game installation detection and validation
//! - Configuration management with TOML serialization
//! - Support for both CTP1 and CTP2
//! - Cross-platform compatibility
//!
//! ## Modules
//!
//! - [`config`] - Configuration management
//! - [`i18n`] - Internationalization support  
//! - [`installation`] - Installation validation and detection
//! - [`settings`] - Settings UI
//! - [`ui`] - User interface components
//! - [`video`] - Video playback functionality

#![allow(
    clippy::literal_string_with_formatting_args,
    reason = "False positive on doc comments"
)]

extern crate alloc;

/// Installation asset extraction module.
pub mod asset_extraction;
/// Canonical application startup used by the thin executable entry point.
pub mod bootstrap;
/// Configuration management module.
pub mod config;
mod dialogs;
/// Gameplay module.
pub mod gameplay;
/// Internationalization module.
pub mod i18n;
/// Installation validation and detection module.
pub mod installation;
/// CTP2 main menu module.
pub mod main_menu;
#[cfg(test)]
/// Shared SDL test serialization.
pub mod sdl_test_lock;
/// Settings UI module.
pub mod settings;
/// User interface module.
pub mod ui;
/// Video playback module.
pub mod video;
