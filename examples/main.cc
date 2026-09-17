#include <cassert>
#include <iostream>
#include "calculator.h"

int main() {
    ICalculator* calc = nullptr;
    CalcStatus ret = WelsCreateCalculator(&calc);
    assert(ret == CALC_OK);
    assert(calc != nullptr);

    assert(WelsCreateCalculator(nullptr) == CALC_ERR_NULL_PTR);

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

    WelsDestroyCalculator(calc);
    std::cout << "All C++ calculator assertions passed!\n";
    return 0;
}
