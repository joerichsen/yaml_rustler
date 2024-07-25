defmodule YamlRustlerTest do
  use ExUnit.Case
  doctest YamlRustler

  test "parses simple YAML" do
    yaml = """
    foo: bar
    baz: 42
    """
    assert {:ok, %{"foo" => "bar", "baz" => 42}} == YamlRustler.parse(yaml)
  end

  test "parses complex YAML" do
    yaml = """
    ---
    name: John Doe
    age: 30
    address:
      street: 123 Main St
      city: Anytown
    hobbies:
      - reading
      - cycling
    """
    expected = %{
      "name" => "John Doe",
      "age" => 30,
      "address" => %{
        "street" => "123 Main St",
        "city" => "Anytown"
      },
      "hobbies" => ["reading", "cycling"]
    }
    assert {:ok, expected} == YamlRustler.parse(yaml)
  end

  test "handles parsing errors" do
    yaml = "invalid: : yaml"
    assert {:error, "YAML parsing error: " <> _} = YamlRustler.parse(yaml)
  end
end
