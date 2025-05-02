#include <concepts>
#include <cstddef>
#include <cstdint>
#define print_result
#ifdef print_result
# include <iostream>
#endif
#include <random>
#include <tuple>
#include <type_traits>
#include <utility>

// clang-format off
template <std::size_t> struct int_selector {};
template <> struct int_selector<1> { using type = std::uint8_t; };
template <> struct int_selector<2> { using type = std::uint16_t; };
template <> struct int_selector<4> { using type = std::uint32_t; };
template <> struct int_selector<8> { using type = std::uint64_t; };
template <typename T> using int_selector_t = typename int_selector<sizeof(T)>::type;
template <typename T> struct fp_min_max {};
template <>
struct fp_min_max<double>
{
    static constexpr double min{-1.797693E+308};
    static constexpr double max{1.797693E+308};
};
template <>
struct fp_min_max<float>
{
    static constexpr float min{-3.402823E38};
    static constexpr float max{3.402823E38};
};
// clang-format on

template <std::floating_point T>
T prepare_arg()
{
    static std::mt19937 gen{std::random_device{}()};
    static std::uniform_real_distribution<T> random_engine{fp_min_max<T>::min, fp_min_max<T>::max};
    return random_engine(gen);
}

template <std::integral T>
T prepare_arg()
{
    static std::mt19937_64 random_engine{std::random_device{}()};
    return static_cast<T>(random_engine());
}

template <std::floating_point T>
bool equal(T lhs, T rhs)
{
    return std::abs(lhs - rhs) < 1e-6 ||
           std::bit_cast<int_selector_t<T>>(lhs) == std::bit_cast<int_selector_t<T>>(rhs) ||
           (std::isnan(lhs) && std::isnan(rhs));
}

template <std::integral T>
constexpr bool equal(T lhs, T rhs)
{
    return lhs == rhs;
}

template <typename T>
constexpr auto trans(T arg)
{
    return static_cast<std::conditional_t<
        std::is_same_v<T, signed char> || std::is_same_v<T, unsigned char>, int, T>>(arg);
}

struct out
{
    template <typename T>
    out &operator<<(const T &arg)
    {
        std::cout << trans(arg) << ' ';
        return *this;
    }
};

out o{};

template <typename R, typename... Args, std::size_t... I>
bool test_impl(R (*f)(Args...), R (*soft_f)(Args...), std::index_sequence<I...>)
{
    auto args{std::make_tuple(prepare_arg<std::tuple_element_t<I, std::tuple<Args...>>>()...)};
    auto op_out{std::apply(f, args)}, soft_op_out{std::apply(soft_f, args)};
#ifdef print_result
    ((o << "args:") << ... << std::get<I>(args)) << '\n';
    std::cout << "    " << trans(op_out) << ", " << trans(soft_op_out) << '\n';
#endif
    return equal(op_out, soft_op_out);
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
