#include <bit>
#include <cmath>
#include <cstddef>
#define print_result
#ifdef print_result
# include <iostream>
#endif
#include <random>
#include <tuple>
#include <type_traits>
#include <utility>

// clang-format off
template <std::size_t> struct int_selector { using type = void; };
template <> struct int_selector<1> { using type = std::uint8_t; };
template <> struct int_selector<2> { using type = std::uint16_t; };
template <> struct int_selector<4> { using type = std::uint32_t; };
template <> struct int_selector<8> { using type = std::uint64_t; };
template <std::size_t N> using int_selector_t = typename int_selector<N>::type;
// clang-format on

template <typename To, typename From>
To prepare_arg(From arg)
{
    using inter_type = int_selector_t<sizeof(To)>;
    return std::bit_cast<To>(static_cast<inter_type>(arg));
}

template <typename R, typename... Args, std::size_t... I>
bool test_impl(R (*f)(Args...), R (*soft_f)(Args...), std::index_sequence<I...>)
{
    static std::mt19937_64 r{std::random_device{}()};
    auto args{std::make_tuple(prepare_arg<std::tuple_element_t<I, std::tuple<Args...>>>(r())...)};
    auto op_out{std::apply(f, args)}, soft_op_out{std::apply(soft_f, args)};
#ifdef print_result
    std::cout << op_out << ", " << soft_op_out << '\n';
#endif
    if constexpr (std::is_floating_point_v<R>)
        return std::abs(op_out - soft_op_out) < 1e-6;
    return op_out == soft_op_out;
}

template <typename R, typename... Args>
bool test(R (*f)(Args...), R (*soft_f)(Args...))
{
    std::make_index_sequence<sizeof...(Args)> index{};
    return test_impl(f, soft_f, index);
}

asm("    .text\n"
    "    .align 1\n"
    "    .globl op\n"
    "    .type op, @function\n"
    "op:\n"
#include "instcode.h"
    "    ret\n");

#include "funcs.c"
extern "C" decltype(soft_op) op;

int main()
{
    for (std::size_t i{0}; i < 100; i++)
    {
        if (!test(op, soft_op))
            return 1;
    }
}
