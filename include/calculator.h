#pragma once
#include "calc_def.h"
#include <cstddef>
#include <cstdint>
#include <type_traits>

#ifdef __clang__
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdollar-in-identifier-extension"
#endif // __clang__

namespace rust {
inline namespace cxxbridge1 {
// #include "rust/cxx.h"

#ifndef CXXBRIDGE1_RUST_OPAQUE
#define CXXBRIDGE1_RUST_OPAQUE
class Opaque {
public:
  Opaque() = delete;
  Opaque(const Opaque &) = delete;
  ~Opaque() = delete;
};
#endif // CXXBRIDGE1_RUST_OPAQUE

#ifndef CXXBRIDGE1_IS_COMPLETE
#define CXXBRIDGE1_IS_COMPLETE
namespace detail {
namespace {
template <typename T, typename = std::size_t>
struct is_complete : std::false_type {};
template <typename T>
struct is_complete<T, decltype(sizeof(T))> : std::true_type {};
} // namespace
} // namespace detail
#endif // CXXBRIDGE1_IS_COMPLETE

#ifndef CXXBRIDGE1_LAYOUT
#define CXXBRIDGE1_LAYOUT
class layout {
  template <typename T>
  friend std::size_t size_of();
  template <typename T>
  friend std::size_t align_of();
  template <typename T>
  static typename std::enable_if<std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_size_of() {
    return T::layout::size();
  }
  template <typename T>
  static typename std::enable_if<!std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_size_of() {
    return sizeof(T);
  }
  template <typename T>
  static
      typename std::enable_if<detail::is_complete<T>::value, std::size_t>::type
      size_of() {
    return do_size_of<T>();
  }
  template <typename T>
  static typename std::enable_if<std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_align_of() {
    return T::layout::align();
  }
  template <typename T>
  static typename std::enable_if<!std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_align_of() {
    return alignof(T);
  }
  template <typename T>
  static
      typename std::enable_if<detail::is_complete<T>::value, std::size_t>::type
      align_of() {
    return do_align_of<T>();
  }
};

template <typename T>
std::size_t size_of() {
  return layout::size_of<T>();
}

template <typename T>
std::size_t align_of() {
  return layout::align_of<T>();
}
#endif // CXXBRIDGE1_LAYOUT
} // namespace cxxbridge1
} // namespace rust

enum class CalcStatus : ::std::int32_t;
struct ICalculator;

#ifndef CXXBRIDGE1_ENUM_CalcStatus
#define CXXBRIDGE1_ENUM_CalcStatus
enum class CalcStatus : ::std::int32_t {
  CALC_OK = 0,
  CALC_ERR_NULL_PTR = -1,
  CALC_ERR_INVALID_OPTION = -2,
};
using enum CalcStatus;
#endif // CXXBRIDGE1_ENUM_CalcStatus

#ifndef CXXBRIDGE1_STRUCT_ICalculator
#define CXXBRIDGE1_STRUCT_ICalculator
struct ICalculator final : public ::rust::Opaque {
  ::CalcStatus Initialize(::SCalcParam const *param) noexcept;
  ::std::int64_t add(::std::int64_t a, ::std::int64_t b) noexcept;
  ::std::int64_t subtract(::std::int64_t a, ::std::int64_t b) noexcept;
  ::std::int64_t multiply(::std::int64_t a, ::std::int64_t b) noexcept;
  ::std::int64_t divide(::std::int64_t a, ::std::int64_t b) noexcept;
  ::CalcStatus SetOption(::CALC_OPTION option_id, ::c_void *option) noexcept;
  ::CalcStatus GetOption(::CALC_OPTION option_id, ::c_void *option) noexcept;
  ~ICalculator() = delete;

private:
  friend ::rust::layout;
  struct layout {
    static ::std::size_t size() noexcept;
    static ::std::size_t align() noexcept;
  };
};
#endif // CXXBRIDGE1_STRUCT_ICalculator

::CalcStatus WelsCreateCalculator(::ICalculator **pp_calc) noexcept;

void WelsDestroyCalculator(::ICalculator *p_calc) noexcept;

#ifdef __clang__
#pragma clang diagnostic pop
#endif // __clang__
