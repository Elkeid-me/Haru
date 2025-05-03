#!elixir

import Enum

defmodule GenTest do
  @op [:+, :-, :*, :/, :%, :&, :|, :^, :"<<", :">>"]
  @int_op MapSet.new([:%, :&, :|, :^, :"<<", :">>"])
  @int_type [
    :"signed char",
    :"unsigned char",
    :short,
    :"unsigned short",
    :int,
    :"unsigned int",
    :long,
    :"unsigned long"
  ]
  @float_type [:float, :double]

  defp exp_tree([x]), do: [x]

  defp exp_tree(list),
    do:
      1..(length(list) - 1)
      |> map(&split(list, &1))
      |> map(fn {l, r} -> {exp_tree(l), exp_tree(r)} end)
      |> flat_map(fn {l, r} -> for left <- l, right <- r, fun <- @op, do: {fun, left, right} end)

  defp transform(x, type_spec) when is_atom(x),
    do: if(Map.get(type_spec, x) in @float_type, do: {x, :float}, else: {x, :int})

  defp transform(x, _type_spec) when is_float(x), do: {{random(@float_type), x}, :float}
  defp transform(x, _type_spec) when is_integer(x), do: {{random(@int_type), x}, :int}

  defp transform({f, x, y}, type_spec) do
    {x_trans, x_trans_type} = transform(x, type_spec)
    {y_trans, y_trans_type} = transform(y, type_spec)

    if f in @int_op do
      case {x_trans_type, y_trans_type} do
        {:int, :int} ->
          {{f, x_trans, y_trans}, :int}

        {:int, :float} ->
          {{f, x_trans, {random(@int_type), y_trans}}, :int}

        {:float, :int} ->
          {{f, {random(@int_type), x_trans}, y_trans}, :int}

        {:float, :float} ->
          {{f, {random(@int_type), x_trans}, {random(@int_type), y_trans}}, :int}
      end
    else
      case {x_trans_type, y_trans_type} do
        {:int, :int} -> {{f, x_trans, y_trans}, :int}
        _ -> {{f, x_trans, y_trans}, :float}
      end
    end
  end

  defp to_str(x) when is_atom(x) or is_float(x) or is_integer(x), do: "#{x}"
  defp to_str({f, x}), do: "(#{f})(#{to_str(x)})"
  defp to_str({f, x, y}), do: "(#{to_str(x)}) #{f} (#{to_str(y)})"

  defp used_paras(x) when is_atom(x), do: MapSet.new([x])
  defp used_paras({_f, x}), do: used_paras(x)
  defp used_paras({_f, x, y}), do: MapSet.union(used_paras(x), used_paras(y))
  defp used_paras(_), do: MapSet.new()

  defp random_type(), do: [random(@int_type), random(@int_type), random(@float_type)] |> random()

  def print_function({{exp_tree, paras, type_spec}, index}) do
    type = random_type()
    used_paras_ = used_paras(exp_tree)

    para_spec =
      paras
      |> uniq()
      |> filter(fn para -> para in used_paras_ end)
      |> map(fn para -> "#{Map.get(type_spec, para)} #{para}" end)
      |> join(", ")

    func_str =
      "[[gnu::noinline]] #{type} f_#{index}(#{para_spec}) { return #{to_str(exp_tree)}; }"

    {"#ifdef use_f_#{index}\n#{func_str}\n#endif", func_str}
  end

  def gen_func(paras) do
    type_spec = %{a: random_type(), b: random_type(), c: random_type()}

    paras
    |> exp_tree()
    |> map(&transform(&1, type_spec))
    |> map(fn exp_tree -> {elem(exp_tree, 0), paras, type_spec} end)
  end
end

args =
  System.argv()
  |> OptionParser.parse(strict: [macro: :string, no_macro: :string])
  |> elem(0)
  |> Map.new()

macro_file = args |> Map.get(:macro) |> File.open!([:write, :utf8])
no_macro_file = args |> Map.get(:no_macro) |> File.open!([:write, :utf8])

random_int = :rand.uniform(114_514_810)
random_float = :rand.uniform() * 114_514_810

paras = [:a, :b, :c, random_int, random_float]

for e_1 <- paras, e_2 <- paras, e_3 <- paras do
  [e_1, e_2, e_3]
end
|> flat_map(&GenTest.gen_func/1)
|> with_index()
|> map(&GenTest.print_function/1)
|> each(fn {macro, no_macro} ->
  IO.puts(macro_file, macro)
  IO.puts(no_macro_file, no_macro)
end)
