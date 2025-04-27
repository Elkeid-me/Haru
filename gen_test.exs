defmodule ExpTree do
  @op [:add, :sub, :mul, :div, :mod, :and, :or, :xor, :shl, :shr]

  def expression_tree([x]), do: [x]

  def expression_tree(list),
    do:
      1..(length(list) - 1)
      |> Enum.map(&Enum.split(list, &1))
      |> Enum.map(fn {left_list, right_list} ->
        {expression_tree(left_list), expression_tree(right_list)}
      end)
      |> Enum.flat_map(fn {left_list, right_list} ->
        for left <- left_list, right <- right_list, fun <- @op, do: {fun, left, right}
      end)
end

defmodule Transform do
  @int_op MapSet.new([:mod, :and, :or, :xor, :shl, :shr])
  @int_type [:i8, :u8, :i16, :u16, :i32, :u32, :i64, :u64]
  @float_type [:f32, :f64]
  @types Enum.concat(@int_type, @float_type)

  def transform(x, type_spec) when is_atom(x) do
    if Map.get(type_spec, x) in @float_type do
      {x, :float}
    else
      {x, :int}
    end
  end

  def transform(x, _type_spec) when is_float(x) do
    type = Enum.random(@float_type)
    {{type, x}, :float}
  end

  def transform(x, _type_spec) when is_integer(x) do
    type = Enum.random(@int_type)
    {{type, x}, :int}
  end

  def transform({f, x, y}, type_spec) do
    {x_transform, x_transform_type} = transform(x, type_spec)
    {y_transform, y_transform_type} = transform(y, type_spec)

    if f in @int_op do
      case {x_transform_type, y_transform_type} do
        {:int, :int} ->
          {{f, x_transform, y_transform}, :int}

        {:int, :float} ->
          {{f, x_transform, {Enum.random(@int_type), y_transform}}, :int}

        {:float, :int} ->
          {{f, {Enum.random(@int_type), x_transform}, y_transform}, :int}

        {:float, :float} ->
          {{f, {Enum.random(@int_type), x_transform}, {Enum.random(@int_type), y_transform}},
           :int}
      end
    else
      case {x_transform_type, y_transform_type} do
        {:int, :int} ->
          {{f, x_transform, y_transform}, :int}

        _ ->
          {{f, x_transform, y_transform}, :float}
      end
    end
  end

  def op_str(op) do
    case op do
      :add -> "+"
      :sub -> "-"
      :mul -> "*"
      :div -> "/"
      :mod -> "%"
      :and -> "&"
      :or -> "|"
      :xor -> "^"
      :shl -> "<<"
      :shr -> ">>"
    end
  end

  def ty_str(ty) do
    case ty do
      :i8 -> "signed char"
      :u8 -> "unsigned char"
      :i16 -> "short"
      :u16 -> "unsigned short"
      :i32 -> "int"
      :u32 -> "unsigned"
      :i64 -> "long"
      :u64 -> "unsigned long"
      :f32 -> "float"
      :f64 -> "double"
    end
  end

  def to_str(x) when is_atom(x) or is_float(x) or is_integer(x), do: "#{x}"
  def to_str({f, x}), do: "(#{ty_str(f)})(#{to_str(x)})"
  def to_str({f, x, y}), do: "(#{to_str(x)}) #{op_str(f)} (#{to_str(y)})"

  def used_paras(x) when is_atom(x), do: MapSet.new([x])
  def used_paras({_f, x}), do: used_paras(x)
  def used_paras({_f, x, y}), do: MapSet.union(used_paras(x), used_paras(y))
  def used_paras(_x), do: MapSet.new()

  def print_function({x, index}, paras, type_spec) do
    type = Enum.random(@types)
    used_paras_ = used_paras(x)

    para_spec =
      paras
      |> Enum.filter(fn para -> para in used_paras_ end)
      |> Enum.map(fn para -> "#{ty_str(Map.get(type_spec, para))} #{para}" end)
      |> Enum.join(", ")

    "#{ty_str(type)} f_#{index}(#{para_spec}) { return #{to_str(x)}; }"
  end
end

random_int = :rand.uniform(114_514)
random_float = :rand.uniform() * 114_514

type_spec = %{a: :f64, b: :f32, c: :i16}
paras = [:a, :b, :c, random_int, random_float]

for e_1 <- paras, e_2 <- paras, e_3 <- paras do
  [e_1, e_2, e_3]
  |> ExpTree.expression_tree()
  |> Enum.map(&Transform.transform(&1, type_spec))
  |> Enum.map(&elem(&1, 0))
end
|> List.flatten()
|> Enum.with_index()
|> Enum.map(&Transform.print_function(&1, paras, type_spec))
|> Enum.join("\n")
|> IO.puts()
