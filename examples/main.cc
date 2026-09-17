#include <cassert>
#include <iostream>
#include "calculator.h"

int main() {
    ICalculator* calc = nullptr;
    CalcStatus ret = WelsCreateCalculator(&calc);
    assert(ret == CALC_OK);
    assert(calc != nullptr);

    assert(WelsCreateCalculator(nullptr) == CALC_ERR_NULL_PTR);

    // Initialize calculator via C struct pointer (mirroring OpenH264's Initialize)
    SCalcParam param = {
        .initial_value = 0,
        .scale_factor = 1,
    };
    assert(calc->Initialize(&param) == CALC_OK);

    int64_t sum = calc->add(10, 5);
    int64_t diff = calc->subtract(10, 5);
    int64_t prod = calc->multiply(10, 5);
    int64_t quot = calc->divide(10, 5);

    std::cout << "10 + 5 = " << sum << "\n";
    std::cout << "10 - 5 = " << diff << "\n";
    std::cout << "10 * 5 = " << prod << "\n";
    std::cout << "10 / 5 = " << quot << "\n";

    assert(sum == 15);
    assert(diff == 5);
    assert(prod == 50);
    assert(quot == 2);

    // Query internal state via void* GetOption (mirroring OpenH264's GetOption)
    int64_t op_count = 0;
    assert(calc->GetOption(CALC_OPTION_OP_COUNT, &op_count) == CALC_OK);
    assert(op_count == 4);

    // Modify runtime option via void* SetOption (mirroring OpenH264's SetOption)
    int64_t new_scale = 2;
    assert(calc->SetOption(CALC_OPTION_SCALE_FACTOR, &new_scale) == CALC_OK);

    int64_t scaled_sum = calc->add(10, 5);
    assert(scaled_sum == 30);

    int64_t last_result = 0;
    assert(calc->GetOption(CALC_OPTION_LAST_RESULT, &last_result) == CALC_OK);
    assert(last_result == 30);

    WelsDestroyCalculator(calc);
    std::cout << "All C++ calculator assertions passed!\n";
    return 0;
}
