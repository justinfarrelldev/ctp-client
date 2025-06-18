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

/// Configuration management module.
pub mod config;
/// Internationalization module.
pub mod i18n;
/// Installation validation and detection module.
pub mod installation;
/// Settings UI module.
pub mod settings;
/// User interface module.
pub mod ui;
