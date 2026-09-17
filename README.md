# rust-toy: C++ Interface Integration via `cxx` (OpenH264 Reference Pattern)

A pure-Rust reference project demonstrating how to expose an existing C++ interface (`ICalculator` and `WelsCreateCalculator` / `WelsDestroyCalculator`, modeled directly on OpenH264's `ISVCEncoder` / `WelsCreateSVCEncoder`) using [`cxx`](https://cxx.rs/) **without** manual C/COM function-pointer vtables or modifying C++ caller code.

---

## Executive Summary: Why `cxx` Opaque Types Beat Manual C-Vtables

When integrating a Rust port of a C++ library like OpenH264 (`openh264-rs`) into Chromium behind a compile-time buildflag (`#if BUILDFLAG(ENABLE_RUST_OPENH264)`), there are two ways to preserve source compatibility (`codec_->EncodeFrame(...)`, `WelsCreateSVCEncoder(&codec)`) at the C++ call site:

1. **The Manual C-Vtable Approach** (used in [CL 8402143](https://chromium-review.git.corp.google.com/c/chromium/src/+/8402143) and [`codec_api.rs`](https://github.com/Djuffin/openh264/blob/rust3/rust/crates/openh264-rs/src/api/codec_api.rs)): Emulating a binary COM vtable at offset `0` (`struct ISVCEncoderVtbl`) with function pointers.
2. **The `cxx` Opaque-Type Approach** (demonstrated in this repository): Declaring `type ISVCEncoder;` as an `extern "Rust"` opaque type in `#[cxx::bridge]`, which autogenerates a C++ class with non-virtual member functions that directly call `extern "C"` Rust bridge symbols.

Because Chromium selects between C++ OpenH264 and Rust OpenH264 at **compile time** via `BUILDFLAG(ENABLE_RUST_OPENH264)` (rather than mixing both implementations behind the same pointer at runtime), emulating a binary vtable at offset `0` provides zero benefit while introducing severe security, correctness, and maintenance costs.

### Comparison Matrix

| Dimension | Manual C-Vtable Approach (CL 8402143) | `cxx` Opaque-Type Approach (`rust-toy`) |
| :--- | :--- | :--- |
| **Call Dispatch** | Indirect call through function pointer (`vtable->Method(this, ...)`) | **Direct call** to `extern "C"` symbol (`cxxbridge1$ISVCEncoder$Method(...)`) |
| **CFI Security** | Requires `DISABLE_CFI_ICALL` on **every method** | **Full CFI protection** (`0` `DISABLE_CFI_ICALL` needed) |
| **Compile-Time Type Safety** | Manual sync between `.h` and `.rs` (prone to silent UB) | **Single source of truth** in `#[cxx::bridge]`; compiler-verified |
| **MiraclePtr / Layout** | Requires `RAW_PTR_EXCLUSION` & `offsetof == 0` | `::rust::Opaque` has **no fields or raw pointers** in C++ |
| **Rust Allocation (`WelsCreate*`)** | 2 heap allocations (`Box<Vtbl>` + `Box<Impl>`) + self-referential pointer | **1 standard `Box::new(ISVCEncoder::new())`** |
| **Method Receiver in Rust** | Raw `*mut ISVCEncoder` downcast via `this as *mut Impl` | **Safe Rust reference (`&mut self`)** passed directly by `cxx` |
| **Caller Syntax in C++** | `codec_->Initialize(...)`, `WelsCreateSVCEncoder(&codec)` | **Identical** (`codec_->Initialize(...)`, `WelsCreateSVCEncoder(&codec)`) |
| **Chromium Build Integration** | Custom handwritten header `openh264_c_api.h` | Native GN support via `cxx_bindings = [ "src/lib.rs" ]` in `rust_static_library.gni` |

---

## Detailed Analysis of Issues in the C-Vtable Approach

### 1. Disables Control Flow Integrity (`DISABLE_CFI_ICALL`) & Masks Real ABI Bugs
In `media/video/openh264_c_api.h` ([CL 8402143](https://chromium-review.git.corp.google.com/c/chromium/src/+/8402143)), every inline method on `ISVCEncoder` dispatches through a C struct of raw function pointers (`vtable->Method(this, ...)`). Because Chromium's Control Flow Integrity (`-fsanitize=cfi-icall`) blocks cross-language indirect calls through C function pointers into Rust, **every single method** had to be annotated with `DISABLE_CFI_ICALL`, weakening browser CFI enforcement on every encoded frame.

> **Concrete Undefined Behavior Bug Found Masked by `DISABLE_CFI_ICALL`:**
> In CL 8402143 (`media/video/openh264_c_api.h` lines 45–47 & 92–94), `ForceIntraFrame` is declared in C++ as taking **3 arguments**:
> ```cpp
> // media/video/openh264_c_api.h (CL 8402143)
> int(MEDIA_OPENH264_CDECL* ForceIntraFrame)(ISVCEncoder*, bool bIDR, int iLayerId);
>
> DISABLE_CFI_ICALL
> inline int ForceIntraFrame(bool bIDR, int iLayerId = -1) {
>   return vtable->ForceIntraFrame(this, bIDR, iLayerId);
> }
> ```
> However, in `openh264-rs` (`src/api/codec_api.rs` lines 1175 & 1922), the Rust vtable slot and implementation only take **2 arguments**:
> ```rust
> // codec_api.rs (openh264-rs)
> pub ForceIntraFrame: unsafe extern "C" fn(pThis: *mut ISVCEncoder, bIDR: bool) -> i32,
>
> unsafe extern "C" fn encoder_force_intra_c(this: *mut ISVCEncoder, bIDR: bool) -> i32
> ```
> Calling a 2-argument `extern "C"` function through a 3-argument function pointer is **Undefined Behavior** in C/C++. Normally, Chromium's `cfi-icall` would immediately trap on this signature mismatch at runtime, but `DISABLE_CFI_ICALL` silenced the check. With `cxx`, this class of bug is impossible because C++ and Rust signatures are generated from a single declaration and verified at compile time.

### 2. MiraclePtr Exclusions & Fragile Layout Coupling on the C++ Side
To mimic OpenH264's C layout, `media/video/openh264_c_api.h` defines:
```cpp
struct ISVCEncoder {
  RAW_PTR_EXCLUSION const ISVCEncoderVtbl* vtable;
  // ...
};
static_assert(std::is_standard_layout_v<ISVCEncoder>);
static_assert(sizeof(ISVCEncoder) == sizeof(void*));
static_assert(offsetof(ISVCEncoder, vtable) == 0);
```
This requires opting out of MiraclePtr (`RAW_PTR_EXCLUSION`) and manually maintaining exact struct offset and calling-convention (`__cdecl` on Windows) parity between C++ and Rust.

### 3. Significant Unsafe Boilerplate on the Rust Side (`codec_api.rs`)
Because Rust does not natively emit C COM vtables, [`codec_api.rs`](https://github.com/Djuffin/openh264/blob/rust3/rust/crates/openh264-rs/src/api/codec_api.rs) has to manually construct and manage them:
- **Double Heap Allocations & Self-Referential Structs (`lines 3714–3741`)**:
  `WelsCreateSVCEncoder` allocates a `Box<ISVCEncoderVtbl>` on the heap, allocates a wrapper struct `Box<CWelsH264SVCEncoderImpl>` containing `base: ISVCEncoder` at offset `0`, sets a self-referential pointer (`enc.base.lpVtbl = &*enc.pVtbl`), and casts `*mut CWelsH264SVCEncoderImpl` to `*mut ISVCEncoder`.
- **19 Manual `unsafe extern "C" fn` Trampolines (`lines 1570–1850`, etc.)**:
  Every method on `ISVCEncoder` and `ISVCDecoder` requires a handwritten `unsafe extern "C" fn` trampoline that checks `if this.is_null()`, casts `this as *mut CWelsH264SVCEncoderImpl` (relying on `#[repr(C)]` first-field subobject layout rules), and forwards to the inner Rust encoder.
- **Duplicate `unsafe` Vtable Dispatchers for Rust Unit Tests (`lines 1203–1269`)**:
  Because `ISVCEncoder` in `codec_api.rs` is defined as a raw vtable struct rather than a normal Rust type, even Rust unit tests inside the crate have to call `unsafe { ((*(*this).lpVtbl).Initialize)(this, pParam) }`.

With `cxx`, **all of this disappears**:
- `ISVCEncoder` in `extern "Rust"` **is** the Rust struct directly (`Box::new(ISVCEncoder::new())`).
- `cxx` generates direct `extern "C"` FFI thunks automatically and passes `self: &mut ISVCEncoder` (a safe, non-null mutable reference) directly to Rust methods.

---

## Architecture of This Reference Project (`rust-toy`)

This repository demonstrates every pattern needed to apply the `cxx` opaque-type approach to `openh264-rs`:

1. **[include/calc_def.h](include/calc_def.h)** *(Existing C/C++ definitions header, analogous to OpenH264's `codec_app_def.h`)*:
   - Defines an external C parameter struct `SCalcParam` (`initial_value`, `scale_factor`).
   - Defines an external C option enum `CALC_OPTION` (`CALC_OPTION_SCALE_FACTOR`, `CALC_OPTION_LAST_RESULT`, `CALC_OPTION_OP_COUNT`).
   - Defines `using c_void = void;` so `cxx` can bind `void*` parameters cleanly across FFI.
2. **[src/lib.rs](src/lib.rs)** *(Pure Rust implementation using `#[cxx::bridge]`)*:
   - Binds `SCalcParam`, `CALC_OPTION`, and `c_void` via `unsafe extern "C++" { include!("calc_def.h"); ... }` and `cxx::ExternType` (`cxx` automatically emits `static_assert(sizeof(SCalcParam) == ...)` in C++ to guarantee layout parity).
   - Defines shared status enum `CalcStatus` (`CALC_OK`, `CALC_ERR_NULL_PTR`, `CALC_ERR_INVALID_OPTION`).
   - Exposes stateful `ICalculator` (`&mut self` receiver) with:
     - `CalcStatus Initialize(const SCalcParam* param)`
     - `int64_t add(int64_t a, int64_t b)`, `subtract`, `multiply`, `divide`
     - `CalcStatus SetOption(CALC_OPTION option_id, void* option)`
     - `CalcStatus GetOption(CALC_OPTION option_id, void* option)`
   - Exposes C-style factory/destructor functions:
     - `CalcStatus WelsCreateCalculator(ICalculator** pp_calc)`
     - `void WelsDestroyCalculator(ICalculator* p_calc)`
3. **[build.rs](build.rs)**:
   - Compiles the C++ bridge with `-std=c++20`.
   - Automatically post-processes `enum class` blocks in the generated header to append C++20 `using enum <EnumName>;` declarations so C++ callers can use unscoped enum constants (`CALC_OK`) without prefixing `CalcStatus::`.
   - Exports the self-contained header to [include/calculator.h](include/calculator.h).
4. **[examples/main.cc](examples/main.cc)** & **[tests/cpp_consumer.rs](tests/cpp_consumer.rs)**:
   - End-to-end C++20 consumer and automated Cargo integration test verifying `WelsCreateCalculator(&calc)`, `calc->Initialize(&param)`, arithmetic methods, and `calc->SetOption(...)` / `calc->GetOption(...)` with `void*`.

---

## Concrete `#[cxx::bridge]` Blueprint for `openh264-rs` in Chromium

To integrate `openh264-rs` into Chromium with **zero changes** to `media/video/openh264_video_encoder.cc` and **zero `DISABLE_CFI_ICALL`**, `openh264-rs` can use the exact pattern from `src/lib.rs`:

```rust
#[cxx::bridge(namespace = "media")]
mod ffi {
    unsafe extern "C++" {
        include!("third_party/openh264/src/codec/api/wels/codec_app_def.h");
        include!("third_party/openh264/src/codec/api/wels/codec_def.h");

        type SEncParamBase = crate::api::codec_api::SEncParamBase;
        type SEncParamExt = crate::api::codec_api::SEncParamExt;
        type SSourcePicture = crate::api::codec_api::SSourcePicture;
        type SFrameBSInfo = crate::api::codec_api::SFrameBSInfo;
        type ENCODER_OPTION = crate::api::codec_api::ENCODER_OPTION;
        type c_void = crate::api::codec_api::c_void;
    }

    extern "Rust" {
        type ISVCEncoder;

        #[cxx_name = "Initialize"]
        unsafe fn initialize(self: &mut ISVCEncoder, param: *const SEncParamBase) -> i32;

        #[cxx_name = "InitializeExt"]
        unsafe fn initialize_ext(self: &mut ISVCEncoder, param: *const SEncParamExt) -> i32;

        #[cxx_name = "GetDefaultParams"]
        unsafe fn get_default_params(self: &mut ISVCEncoder, param: *mut SEncParamExt) -> i32;

        #[cxx_name = "Uninitialize"]
        fn uninitialize(self: &mut ISVCEncoder) -> i32;

        #[cxx_name = "EncodeFrame"]
        unsafe fn encode_frame(
            self: &mut ISVCEncoder,
            src_pic: *const SSourcePicture,
            bs_info: *mut SFrameBSInfo,
        ) -> i32;

        #[cxx_name = "EncodeParameterSets"]
        unsafe fn encode_parameter_sets(self: &mut ISVCEncoder, bs_info: *mut SFrameBSInfo) -> i32;

        #[cxx_name = "ForceIntraFrame"]
        fn force_intra_frame(self: &mut ISVCEncoder, idr: bool, layer_id: i32) -> i32;

        #[cxx_name = "SetOption"]
        unsafe fn set_option(
            self: &mut ISVCEncoder,
            option_id: ENCODER_OPTION,
            option: *mut c_void,
        ) -> i32;

        #[cxx_name = "GetOption"]
        unsafe fn get_option(
            self: &mut ISVCEncoder,
            option_id: ENCODER_OPTION,
            option: *mut c_void,
        ) -> i32;

        #[cxx_name = "WelsCreateSVCEncoder"]
        unsafe fn wels_create_svc_encoder(pp_encoder: *mut *mut ISVCEncoder) -> i32;

        #[cxx_name = "WelsDestroySVCEncoder"]
        unsafe fn wels_destroy_svc_encoder(p_encoder: *mut ISVCEncoder);
    }
}
```

---

## Building and Testing This Project

Run all Rust unit tests and the automated end-to-end C++20 integration test:

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
