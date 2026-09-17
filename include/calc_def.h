#pragma once

#include <cstdint>

using c_void = void;

// Existing C parameter struct (analogous to OpenH264's SEncParamBase / SSourcePicture)
struct SCalcParam {
  int64_t initial_value;
  int64_t scale_factor;
};

// Existing C option enum (analogous to OpenH264's ENCODER_OPTION)
enum CALC_OPTION : int32_t {
  CALC_OPTION_SCALE_FACTOR = 0,
  CALC_OPTION_LAST_RESULT = 1,
  CALC_OPTION_OP_COUNT = 2,
};
