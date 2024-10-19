
## How to run tests

**1. Runt test in a file:**

```bash
cargo test --test service_test
```

This will run only the tests in the `tests/service_test.rs` file.

**2. Specify test function name:**

If you only want to run specific tests in the `service_test.rs` file, you can specify the function name:

```bash
cargo test --test service_test test_deepseek  // Only run the test_deepseek function
cargo test --test service_test test_glm   // Only run the test_glm function
```

**3. Use filters:**

Run all tests that contain "deepseek" in their names, including tests in service_test.rs
```bash
cargo test deepseek
```

**4. Use the `package` parameter (if your project is a library):**

```bash
cargo test --package lumos --test service_test
```
