use std::env;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_cpp_consumer_integration() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    // Ensure the staticlib is built
    let status = Command::new(&cargo)
        .args(["build", "--offline"])
        .current_dir(&manifest_dir)
        .status()
        .expect("Failed to execute cargo build");
    assert!(status.success(), "cargo build failed");

    let static_lib = manifest_dir.join("target/debug/librust_toy.a");
    assert!(
        static_lib.exists(),
        "Static library not found at {:?}",
        static_lib
    );

    let include_dir = manifest_dir.join("include");
    let cpp_source = manifest_dir.join("examples/main.cc");
    let output_bin = manifest_dir.join("target/debug/cpp_consumer_test_bin");

    let compiler = env::var("CXX").unwrap_or_else(|_| "clang++".to_string());
    let compile_status = Command::new(&compiler)
        .arg("-std=c++20")
        .arg(format!("-I{}", include_dir.display()))
        .arg(&cpp_source)
        .arg(&static_lib)
        .arg("-lpthread")
        .arg("-ldl")
        .arg("-o")
        .arg(&output_bin)
        .status()
        .expect("Failed to invoke C++ compiler");
    assert!(compile_status.success(), "C++ compilation failed");

    let run_output = Command::new(&output_bin)
        .output()
        .expect("Failed to run compiled C++ consumer binary");
    assert!(
        run_output.status.success(),
        "C++ binary exited with failure: {}",
        String::from_utf8_lossy(&run_output.stderr)
    );

    let stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(stdout.contains("10 + 5 = 15"));
    assert!(stdout.contains("10 - 5 = 5"));
    assert!(stdout.contains("10 * 5 = 50"));
    assert!(stdout.contains("10 / 5 = 2"));
    assert!(stdout.contains("All C++ calculator assertions passed!"));
}
