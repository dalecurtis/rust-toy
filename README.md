# rust-toy

A pure-Rust library exposing a C++20 API (`ICalculator` and `WelsCreateCalculator` / `WelsDestroyCalculator`) modeled on OpenH264's `ISVCEncoder` / `WelsCreateSVCEncoder` interface.

## Architecture

- **[include/calc_def.h](file:///usr/local/google/home/dalecurtis/code/rust-toy/include/calc_def.h)**: Existing C/C++ definitions header (analogous to OpenH264's `codec_app_def.h`), defining:
  - `SCalcParam`: C parameter struct passed by pointer to `Initialize`.
  - `CALC_OPTION`: Option enum (`CALC_OPTION_SCALE_FACTOR`, `CALC_OPTION_LAST_RESULT`, `CALC_OPTION_OP_COUNT`).
  - `c_void`: Typedef (`using c_void = void;`) enabling `void*` parameters across `cxx`.
- **[src/lib.rs](file:///usr/local/google/home/dalecurtis/code/rust-toy/src/lib.rs)**: Pure Rust implementation using `#[cxx::bridge]` to expose:
  - `CalcStatus`: Shared status enum (`CALC_OK`, `CALC_ERR_NULL_PTR`, `CALC_ERR_INVALID_OPTION`).
  - `ICalculator`: Stateful opaque C++ class/struct with methods:
    - `CalcStatus Initialize(const SCalcParam* param)`
    - `int64_t add(int64_t a, int64_t b)`, `subtract`, `multiply`, and `divide` (`&mut self`)
    - `CalcStatus SetOption(CALC_OPTION option_id, void* option)`
    - `CalcStatus GetOption(CALC_OPTION option_id, void* option)`
  - `CalcStatus WelsCreateCalculator(ICalculator** pp_calc)`: C-style factory function returning an `ICalculator*` via out-pointer.
  - `void WelsDestroyCalculator(ICalculator* p_calc)`: C-style destructor function to free the instance.
- **[build.rs](file:///usr/local/google/home/dalecurtis/code/rust-toy/build.rs)**: Uses `cxx-build` (`-std=c++20`) to compile the C++ bridge, automatically appends C++20 `using enum <EnumName>;` declarations so enum constants can be used unscoped, and exports the header to `include/calculator.h`.
- **[examples/main.cc](file:///usr/local/google/home/dalecurtis/code/rust-toy/examples/main.cc)**: Example C++20 consumer demonstrating `WelsCreateCalculator(&calc)`, `calc->Initialize(&param)`, arithmetic operations, and `calc->SetOption(...)` / `calc->GetOption(...)` with `void*`.
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
