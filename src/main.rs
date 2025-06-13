fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_function() {
        // Test that main function runs without panicking
        main();
    }

    #[test]
    fn test_hello_world_output() {
        // This test ensures we have coverage of the main function logic
        // In a real application, you would test specific functionality
        // For now, we're just ensuring the main function can be called
        main();
    }
}
