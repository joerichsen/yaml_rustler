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

  test "parses strings" do
    yaml = """
    single_quoted: 'This is a single-quoted string'
    double_quoted: "This is a double-quoted string"
    unquoted: This is an unquoted string
    multiline: |
      This is a
      multiline string
    folded: >
      This is a folded
      string
    """

    assert {:ok, result} = YamlRustler.parse(yaml)
    assert result["single_quoted"] == "This is a single-quoted string"
    assert result["double_quoted"] == "This is a double-quoted string"
    assert result["unquoted"] == "This is an unquoted string"
    assert result["multiline"] == "This is a\nmultiline string\n"
    assert result["folded"] == "This is a folded string\n"
  end

  test "parses numbers" do
    yaml = """
    integer: 42
    negative: -17
    float: 3.14
    scientific: 6.02e23
    """

    assert {:ok, result} = YamlRustler.parse(yaml)
    assert result["integer"] == 42
    assert result["negative"] == -17
    assert result["float"] == 3.14
    assert result["scientific"] == 6.02e23
  end

  test "parses booleans and null" do
    yaml = """
    true_value: true
    false_value: false
    null_value: null
    """

    assert {:ok, result} = YamlRustler.parse(yaml)
    assert result["true_value"] == true
    assert result["false_value"] == false
    assert result["null_value"] == nil
  end

  test "parses lists" do
    yaml = """
    simple_list:
      - item1
      - item2
      - item3
    nested_list:
      - - nested1
        - nested2
      - - nested3
        - nested4
    """

    assert {:ok, result} = YamlRustler.parse(yaml)
    assert result["simple_list"] == ["item1", "item2", "item3"]
    assert result["nested_list"] == [["nested1", "nested2"], ["nested3", "nested4"]]
  end

  test "parses maps" do
    yaml = """
    simple_map:
      key1: value1
      key2: value2
    nested_map:
      outer1:
        inner1: value1
        inner2: value2
      outer2:
        inner3: value3
    """

    assert {:ok, result} = YamlRustler.parse(yaml)
    assert result["simple_map"] == %{"key1" => "value1", "key2" => "value2"}

    assert result["nested_map"] == %{
             "outer1" => %{"inner1" => "value1", "inner2" => "value2"},
             "outer2" => %{"inner3" => "value3"}
           }
  end

  test "parses anchors and aliases" do
    yaml = """
    defaults: &defaults
      adapter: postgres
      host: localhost

    development:
      <<: *defaults
      database: myapp_development

    test:
      <<: *defaults
      database: myapp_test
    """

    assert {:ok, result} = YamlRustler.parse(yaml)
    assert result["development"]["adapter"] == "postgres"
    assert result["development"]["host"] == "localhost"
    assert result["development"]["database"] == "myapp_development"
    assert result["test"]["adapter"] == "postgres"
    assert result["test"]["host"] == "localhost"
    assert result["test"]["database"] == "myapp_test"
  end

  test "parses tags" do
    yaml = """
    date: !!timestamp 2023-04-13
    set: !!set
      ? item1
      ? item2
      ? item3
    """

    assert {:ok, result} = YamlRustler.parse(yaml)
    assert result["date"] == "2023-04-13"
    assert is_map(result["set"])
    assert Map.keys(result["set"]) |> Enum.sort() == ["item1", "item2", "item3"]
  end
end
