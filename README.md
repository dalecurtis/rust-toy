# rust-toy

A pure-Rust library exposing a C++ API (`ICalculator` and `WelsCreateCalculator` / `WelsDestroyCalculator`) for integer calculations (`add`, `subtract`, `multiply`, `divide`).

## Architecture

- **[src/lib.rs](file:///usr/local/google/home/dalecurtis/code/rust-toy/src/lib.rs)**: Pure Rust implementation using `#[cxx::bridge]` to expose:
  - `CalcStatus`: Shared enum (`CALC_OK`, `CALC_ERR_NULL_PTR`).
  - `ICalculator`: Opaque C++ class/struct with methods `int64_t add(int64_t a, int64_t b)`, `subtract`, `multiply`, and `divide`.
  - `CalcStatus WelsCreateCalculator(ICalculator** pp_calc)`: C-style factory function returning an `ICalculator*` via out-pointer.
  - `void WelsDestroyCalculator(ICalculator* p_calc)`: C-style destructor function to free the instance.
- **[build.rs](file:///usr/local/google/home/dalecurtis/code/rust-toy/build.rs)**: Uses `cxx-build` (`-std=c++20`) to compile the C++ bridge, automatically appends C++20 `using enum <EnumName>;` declarations so enum constants can be used unscoped, and exports the header to `include/calculator.h`.
- **[examples/main.cc](file:///usr/local/google/home/dalecurtis/code/rust-toy/examples/main.cc)**: Example C++20 consumer demonstrating `WelsCreateCalculator(&calc) == CALC_OK` and calling `calc->add(...)`, `subtract(...)`, `multiply(...)`, and `divide(...)`.
- **[tests/cpp_consumer.rs](file:///usr/local/google/home/dalecurtis/code/rust-toy/tests/cpp_consumer.rs)**: Automated integration test that builds the static library, compiles `examples/main.cc` with `clang++ -std=c++20`, and verifies execution.

## Building and Testing

Run all Rust unit tests and the end-to-end C++ consumer integration test:

```bash
cargo test
```

Build the static library (`target/debug/librust_toy.a`) and autogenerate `include/calculator.h`:

```bash
cargo build
```

Manually compile and run the C++ example:

```bash
clang++ -std=c++20 -Iinclude examples/main.cc target/debug/librust_toy.a -lpthread -ldl -o target/debug/cpp_consumer
./target/debug/cpp_consumer
```
