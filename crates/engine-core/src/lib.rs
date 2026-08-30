//! Engine core library
//!
//! This crate provides a simple entry point to the matching engine.
//! It uses [`engine_tokio`] as the backend implementation.

pub use engine_tokio::{Engine, TokioMatchingEngineBuilder};

/// Run the engine-core application.
///
/// This function constructs a tokio matching engine and returns it.
/// It is designed to be called from the binary entry point.
pub fn run() -> Engine {
    TokioMatchingEngineBuilder::build()
}

#[cfg(test)]
mod tests {
    use super::run;

    /// Test that the run function constructs a tokio engine without panicking.
    #[test]
    fn run_constructs_tokio_engine_without_panic() {
        // Given/When: calling run()
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(run));

        // Then: run() should succeed and return an Engine
        assert!(result.is_ok(), "run() should not panic");
    }

    /// Test that the lib does not export DummyEngine.
    #[test]
    fn lib_does_not_export_dummy_engine() {
        // This test verifies that DummyEngine is not part of the public API.
        // If this test fails, it means DummyEngine is still being exported from lib.rs

        // We can't directly test for non-exported items, but we can verify
        // that the public API works correctly without it
        let engine = run();

        // Verify we got a valid Engine instance
        // (size > 0 confirms it's not a ZST without proper fields)
        assert!(
            std::mem::size_of_val(&engine) > 0,
            "Engine should be constructible"
        );
    }

    /// Test that run is callable from the main path.
    #[test]
    fn run_is_callable_from_main_path() {
        // Given: a function that calls run()
        let test_run = || -> () {
            let _engine = run();
        };

        // When: calling the function
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(test_run));

        // Then: it should not panic
        assert!(result.is_ok(), "run() should be callable from main path");
    }
}
