//! Thin executable entry point; startup and all shared modules belong to the library.

/// Delegates native application startup without compiling a second module tree.
fn main() {
    ctp_client::bootstrap::run();
}
