import Enum

defmodule GenTest do
  @op [:+, :-, :*, :/, :/, :&, :|, :^, :"<<", :">>"]
  @int_op MapSet.new([:/, :&, :|, :^, :"<<", :">>"])
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
  @types concat(@int_type, @float_type)

  def exp_tree([x]), do: [x]

  def exp_tree(list),
    do:
      1..(length(list) - 1)
      |> map(&split(list, &1))
      |> map(fn {l, r} -> {exp_tree(l), exp_tree(r)} end)
      |> flat_map(fn {l, r} -> for left <- l, right <- r, fun <- @op, do: {fun, left, right} end)

  def transform(x, type_spec) when is_atom(x),
    do: if(Map.get(type_spec, x) in @float_type, do: {x, :float}, else: {x, :int})

  def transform(x, _type_spec) when is_float(x), do: {{random(@float_type), x}, :float}
  def transform(x, _type_spec) when is_integer(x), do: {{random(@int_type), x}, :int}

  def transform({f, x, y}, type_spec) do
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

  def to_str(x) when is_atom(x) or is_float(x) or is_integer(x), do: "#{x}"
  def to_str({f, x}), do: "(#{f})(#{to_str(x)})"
  def to_str({f, x, y}), do: "(#{to_str(x)}) #{f} (#{to_str(y)})"

  def used_paras(x) when is_atom(x), do: MapSet.new([x])
  def used_paras({_f, x}), do: used_paras(x)
  def used_paras({_f, x, y}), do: MapSet.union(used_paras(x), used_paras(y))
  def used_paras(_), do: MapSet.new()

  def print_function({x, index}, paras, type_spec) do
    type = random(@types)
    used_paras_ = used_paras(x)

    para_spec =
      paras
      |> filter(fn para -> para in used_paras_ end)
      |> map(fn para -> "#{Map.get(type_spec, para)} #{para}" end)
      |> join(", ")

    "#ifdef use_f_#{index}\n[[gnu::noinline]] #{type} f_#{index}(#{para_spec}) { return #{to_str(x)}; }\n#endif"
  end
end

random_int = :rand.uniform(114_514_810)
random_float = :rand.uniform() * 114_514_810

type_spec = %{a: :double, b: :single, c: :short}
paras = [:a, :b, :c, random_int, random_float]

for e_1 <- paras, e_2 <- paras, e_3 <- paras do
  [e_1, e_2, e_3]
  |> GenTest.exp_tree()
  |> map(&GenTest.transform(&1, type_spec))
  |> map(&elem(&1, 0))
end
|> List.flatten()
|> with_index()
|> map(&GenTest.print_function(&1, paras, type_spec))
|> each(&IO.puts/1)
