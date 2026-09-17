#[cxx::bridge]
mod ffi {
    #[derive(Debug, PartialEq, Eq)]
    #[repr(i32)]
    enum CalcStatus {
        CALC_OK = 0,
        CALC_ERR_NULL_PTR = -1,
    }

    extern "Rust" {
        type ICalculator;

        fn add(self: &ICalculator, a: i64, b: i64) -> i64;
        fn subtract(self: &ICalculator, a: i64, b: i64) -> i64;
        fn multiply(self: &ICalculator, a: i64, b: i64) -> i64;
        fn divide(self: &ICalculator, a: i64, b: i64) -> i64;

        #[cxx_name = "WelsCreateCalculator"]
        unsafe fn wels_create_calculator(pp_calc: *mut *mut ICalculator) -> CalcStatus;

        #[cxx_name = "WelsDestroyCalculator"]
        unsafe fn wels_destroy_calculator(p_calc: *mut ICalculator);
    }
}

/// Pure Rust implementation of `ICalculator`.
/// Uses a non-zero size (`_private: u8`) so heap allocations via `Box::into_raw`
/// produce distinct, well-aligned heap pointers across FFI.
#[repr(C)]
pub struct ICalculator {
    _private: u8,
}

impl Default for ICalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl ICalculator {
    pub fn new() -> Self {
        Self { _private: 0 }
    }

    pub fn add(&self, a: i64, b: i64) -> i64 {
        a.wrapping_add(b)
    }

    pub fn subtract(&self, a: i64, b: i64) -> i64 {
        a.wrapping_sub(b)
    }

    pub fn multiply(&self, a: i64, b: i64) -> i64 {
        a.wrapping_mul(b)
    }

    pub fn divide(&self, a: i64, b: i64) -> i64 {
        if b == 0 {
            0
        } else {
            a.wrapping_div(b)
        }
    }
}

/// C-style factory function to allocate and return an `ICalculator` instance via an out-pointer.
/// Returns `CALC_OK` on success, or `CALC_ERR_NULL_PTR` if `pp_calc` is null.
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
    fn test_calculator_operations() {
        let mut calc_ptr: *mut ICalculator = ptr::null_mut();
        unsafe {
            assert_eq!(
                wels_create_calculator(&mut calc_ptr),
                ffi::CalcStatus::CALC_OK
            );
            assert!(!calc_ptr.is_null());

            let calc = &*calc_ptr;
            assert_eq!(calc.add(15, 27), 42);
            assert_eq!(calc.subtract(50, 8), 42);
            assert_eq!(calc.multiply(6, 7), 42);
            assert_eq!(calc.divide(84, 2), 42);
            assert_eq!(calc.divide(84, 0), 0);

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
