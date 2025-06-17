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
//! - [`installation`] - Installation validation and detection
//! - [`ui`] - User interface components

/// Configuration management module.
pub mod config;
/// Installation validation and detection module.
pub mod installation;
/// User interface module.
pub mod ui;
