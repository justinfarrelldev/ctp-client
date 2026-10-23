//! Fileless architecture guards against compiling the library's modules twice.

/// Architecture regressions are compiled only by the integration test harness.
#[cfg(test)]
mod tests {
    /// The executable must call the shared entry point instead of owning a second tree.
    #[test]
    fn executable_delegates_to_library_without_redeclaring_modules() {
        let source = include_str!("../src/main.rs");
        assert!(
            source.contains("ctp_client::bootstrap::run();"),
            "the executable must invoke the library-owned startup workflow"
        );
        for line in source.lines().map(str::trim) {
            assert!(
                !line.starts_with("mod ")
                    && !line.starts_with("pub mod ")
                    && !line.starts_with("#[path")
                    && !line.contains("include!("),
                "the binary must not compile a second copy of shared modules: {line}"
            );
        }
    }

    /// Deduplication must not hide startup regressions by disabling a test target.
    #[test]
    fn library_and_binary_keep_their_default_test_harnesses() {
        let manifest: toml::Value = toml::from_str(include_str!("../Cargo.toml"))
            .expect("the committed Cargo manifest must be valid TOML");
        let library = manifest.get("lib").expect("explicit canonical library");
        let binaries = manifest
            .get("bin")
            .and_then(toml::Value::as_array)
            .expect("explicit executable target");
        for target in core::iter::once(library).chain(binaries) {
            for flag in ["test", "harness"] {
                assert_ne!(
                    target.get(flag).and_then(toml::Value::as_bool),
                    Some(false),
                    "keep {flag} enabled; move startup tests rather than disabling their target"
                );
            }
        }
    }
}
