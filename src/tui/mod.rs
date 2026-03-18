//! TUI module for terminal user interface.
//!
//! This module provides the interactive terminal user interface for dirloom,
//! built with ratatui and crossterm.

pub mod app;
pub mod dir_picker;
pub mod help;
pub mod log_buffer;
pub mod ui;

pub use app::App;
