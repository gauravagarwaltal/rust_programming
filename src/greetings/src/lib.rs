pub fn hello() -> String {
    //! This returns Hello, world! String
    ("Hello, world!").to_string()
}

// 01. Tests for hello()
#[test] // indicates that this is a test function
fn test_hello() {
    assert_eq!(hello(), "Hello, world!");
}

// 02. Tests for hello(), Idiomatic way
#[cfg(test)] // only compiles when runing tests
mod tests {
    // seperates tests from code
    use super::hello; // import root hello function

    #[test]
    fn test_hello() {
        assert_eq!(hello(), "Hello, world!");
    }
}
