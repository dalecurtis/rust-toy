#[cxx::bridge]
mod ffi {
    #[derive(Debug, PartialEq, Eq)]
    #[repr(i32)]
    enum CalcStatus {
        CALC_OK = 0,
        CALC_ERR_NULL_PTR = -1,
        CALC_ERR_INVALID_OPTION = -2,
    }

    // Bind existing C structs, enums, and void* from include/calc_def.h
    unsafe extern "C++" {
        include!("calc_def.h");

        type SCalcParam = crate::SCalcParam;
        type CALC_OPTION = crate::CALC_OPTION;
        type c_void = crate::c_void;
    }

    extern "Rust" {
        type ICalculator;

        #[cxx_name = "Initialize"]
        unsafe fn initialize(self: &mut ICalculator, param: *const SCalcParam) -> CalcStatus;

        fn add(self: &mut ICalculator, a: i64, b: i64) -> i64;
        fn subtract(self: &mut ICalculator, a: i64, b: i64) -> i64;
        fn multiply(self: &mut ICalculator, a: i64, b: i64) -> i64;
        fn divide(self: &mut ICalculator, a: i64, b: i64) -> i64;

        #[cxx_name = "SetOption"]
        unsafe fn set_option(
            self: &mut ICalculator,
            option_id: CALC_OPTION,
            option: *mut c_void,
        ) -> CalcStatus;

        #[cxx_name = "GetOption"]
        unsafe fn get_option(
            self: &mut ICalculator,
            option_id: CALC_OPTION,
            option: *mut c_void,
        ) -> CalcStatus;

        #[cxx_name = "WelsCreateCalculator"]
        unsafe fn wels_create_calculator(pp_calc: *mut *mut ICalculator) -> CalcStatus;

        #[cxx_name = "WelsDestroyCalculator"]
        unsafe fn wels_destroy_calculator(p_calc: *mut ICalculator);
    }
}

/// Rust representation of external C struct `SCalcParam` defined in `include/calc_def.h`.
/// `cxx` emits compile-time `static_assert(sizeof(SCalcParam) == ...)` to verify exact ABI match.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SCalcParam {
    pub initial_value: i64,
    pub scale_factor: i64,
}

unsafe impl cxx::ExternType for SCalcParam {
    type Id = cxx::type_id!("SCalcParam");
    type Kind = cxx::kind::Trivial;
}

/// Rust representation of external C enum `CALC_OPTION` defined in `include/calc_def.h`.
#[allow(non_camel_case_types)]
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CALC_OPTION {
    CALC_OPTION_SCALE_FACTOR = 0,
    CALC_OPTION_LAST_RESULT = 1,
    CALC_OPTION_OP_COUNT = 2,
}

unsafe impl cxx::ExternType for CALC_OPTION {
    type Id = cxx::type_id!("CALC_OPTION");
    type Kind = cxx::kind::Trivial;
}

/// Opaque type representing C `void` for `void*` (`*mut c_void`) FFI parameters.
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct c_void {
    _private: [u8; 0],
}

unsafe impl cxx::ExternType for c_void {
    type Id = cxx::type_id!("c_void");
    type Kind = cxx::kind::Opaque;
}

/// Pure Rust stateful implementation of `ICalculator`.
pub struct ICalculator {
    scale_factor: i64,
    last_result: i64,
    op_count: i64,
}

impl Default for ICalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl ICalculator {
    pub fn new() -> Self {
        Self {
            scale_factor: 1,
            last_result: 0,
            op_count: 0,
        }
    }

    pub unsafe fn initialize(&mut self, param: *const SCalcParam) -> ffi::CalcStatus {
        let Some(param) = param.as_ref() else {
            return ffi::CalcStatus::CALC_ERR_NULL_PTR;
        };
        self.last_result = param.initial_value;
        self.scale_factor = if param.scale_factor == 0 {
            1
        } else {
            param.scale_factor
        };
        self.op_count = 0;
        ffi::CalcStatus::CALC_OK
    }

    fn record(&mut self, val: i64) -> i64 {
        let scaled = val.wrapping_mul(self.scale_factor);
        self.last_result = scaled;
        self.op_count = self.op_count.wrapping_add(1);
        scaled
    }

    pub fn add(&mut self, a: i64, b: i64) -> i64 {
        self.record(a.wrapping_add(b))
    }

    pub fn subtract(&mut self, a: i64, b: i64) -> i64 {
        self.record(a.wrapping_sub(b))
    }

    pub fn multiply(&mut self, a: i64, b: i64) -> i64 {
        self.record(a.wrapping_mul(b))
    }

    pub fn divide(&mut self, a: i64, b: i64) -> i64 {
        let res = if b == 0 { 0 } else { a.wrapping_div(b) };
        self.record(res)
    }

    pub unsafe fn set_option(
        &mut self,
        option_id: CALC_OPTION,
        option: *mut c_void,
    ) -> ffi::CalcStatus {
        if option.is_null() {
            return ffi::CalcStatus::CALC_ERR_NULL_PTR;
        }
        match option_id {
            CALC_OPTION::CALC_OPTION_SCALE_FACTOR => {
                let val = *(option as *const i64);
                self.scale_factor = if val == 0 { 1 } else { val };
                ffi::CalcStatus::CALC_OK
            }
            _ => ffi::CalcStatus::CALC_ERR_INVALID_OPTION,
        }
    }

    pub unsafe fn get_option(
        &mut self,
        option_id: CALC_OPTION,
        option: *mut c_void,
    ) -> ffi::CalcStatus {
        if option.is_null() {
            return ffi::CalcStatus::CALC_ERR_NULL_PTR;
        }
        match option_id {
            CALC_OPTION::CALC_OPTION_SCALE_FACTOR => {
                *(option as *mut i64) = self.scale_factor;
                ffi::CalcStatus::CALC_OK
            }
            CALC_OPTION::CALC_OPTION_LAST_RESULT => {
                *(option as *mut i64) = self.last_result;
                ffi::CalcStatus::CALC_OK
            }
            CALC_OPTION::CALC_OPTION_OP_COUNT => {
                *(option as *mut i64) = self.op_count;
                ffi::CalcStatus::CALC_OK
            }
        }
    }
}

/// C-style factory function to allocate and return an `ICalculator` instance via an out-pointer.
pub unsafe fn wels_create_calculator(pp_calc: *mut *mut ICalculator) -> ffi::CalcStatus {
    if pp_calc.is_null() {
        return ffi::CalcStatus::CALC_ERR_NULL_PTR;
    }
    let calc = Box::new(ICalculator::new());
    *pp_calc = Box::into_raw(calc);
    ffi::CalcStatus::CALC_OK
}

/// C-style destructor function to free an `ICalculator` instance previously allocated
/// by `WelsCreateCalculator`.
pub unsafe fn wels_destroy_calculator(p_calc: *mut ICalculator) {
    if !p_calc.is_null() {
        let _ = Box::from_raw(p_calc);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn test_calculator_operations_and_options() {
        let mut calc_ptr: *mut ICalculator = ptr::null_mut();
        unsafe {
            assert_eq!(
                wels_create_calculator(&mut calc_ptr),
                ffi::CalcStatus::CALC_OK
            );
            assert!(!calc_ptr.is_null());

            let calc = &mut *calc_ptr;
            let param = SCalcParam {
                initial_value: 100,
                scale_factor: 1,
            };
            assert_eq!(calc.initialize(&param), ffi::CalcStatus::CALC_OK);

            assert_eq!(calc.add(15, 27), 42);
            assert_eq!(calc.subtract(50, 8), 42);
            assert_eq!(calc.multiply(6, 7), 42);
            assert_eq!(calc.divide(84, 2), 42);

            let mut op_count: i64 = 0;
            assert_eq!(
                calc.get_option(
                    CALC_OPTION::CALC_OPTION_OP_COUNT,
                    (&mut op_count as *mut i64).cast::<c_void>()
                ),
                ffi::CalcStatus::CALC_OK
            );
            assert_eq!(op_count, 4);

            let new_scale: i64 = 2;
            assert_eq!(
                calc.set_option(
                    CALC_OPTION::CALC_OPTION_SCALE_FACTOR,
                    (&new_scale as *const i64 as *mut i64).cast::<c_void>()
                ),
                ffi::CalcStatus::CALC_OK
            );
            assert_eq!(calc.add(10, 5), 30);

            wels_destroy_calculator(calc_ptr);
        }
    }

    #[test]
    fn test_null_safety() {
        unsafe {
            assert_eq!(
                wels_create_calculator(ptr::null_mut()),
                ffi::CalcStatus::CALC_ERR_NULL_PTR
            );
            wels_destroy_calculator(ptr::null_mut());
        }
    }
}
