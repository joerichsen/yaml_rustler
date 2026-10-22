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

  test "parses nested anchors and aliases" do
    yaml = """
    defaults: &defaults
      adapter: postgres
      host: localhost
      port: &port 5432

    development:
      <<: *defaults
      database: myapp_development
      port: *port

    production:
      <<: *defaults
      database: myapp_production
      host: prod.example.com
    """

    assert {:ok, result} = YamlRustler.parse(yaml)
    assert result["development"]["adapter"] == "postgres"
    assert result["development"]["host"] == "localhost"
    assert result["development"]["port"] == 5432
    assert result["development"]["database"] == "myapp_development"
    assert result["production"]["adapter"] == "postgres"
    assert result["production"]["host"] == "prod.example.com"
    assert result["production"]["port"] == 5432
    assert result["production"]["database"] == "myapp_production"
  end

  test "parses multiple documents" do
    yaml = """
    ---
    document: 1
    ---
    document: 2
    """

    assert {:ok, result} = YamlRustler.parse(yaml)
    assert result == %{"document" => 1}
  end

  test "parses complex nested structures" do
    yaml = """
    company:
      name: Acme Corp
      employees:
        - name: John Doe
          position: Developer
          skills:
            - Ruby
            - Elixir
        - name: Jane Smith
          position: Designer
          skills:
            - Photoshop
            - Illustrator
      locations:
        headquarters:
          city: New York
          country: USA
        branch:
          city: London
          country: UK
    """

    assert {:ok, result} = YamlRustler.parse(yaml)
    assert result["company"]["name"] == "Acme Corp"
    assert length(result["company"]["employees"]) == 2
    assert hd(result["company"]["employees"])["name"] == "John Doe"
    assert hd(result["company"]["employees"])["skills"] == ["Ruby", "Elixir"]
    assert result["company"]["locations"]["headquarters"]["city"] == "New York"
    assert result["company"]["locations"]["branch"]["country"] == "UK"
  end

  test "parses YAML with various scalar types" do
    yaml = """
    string: Hello, World!
    integer: 42
    float: 3.14159
    scientific: 6.02e23
    boolean_true: true
    boolean_false: false
    null_value: null
    date: 2023-04-13
    time: 13:45:30
    datetime: 2023-04-13T13:45:30Z
    """

    assert {:ok, result} = YamlRustler.parse(yaml)
    assert result["string"] == "Hello, World!"
    assert result["integer"] == 42
    assert_in_delta result["float"], 3.14159, 0.00001
    assert result["scientific"] == 6.02e23
    assert result["boolean_true"] == true
    assert result["boolean_false"] == false
    assert result["null_value"] == nil
    assert result["date"] == "2023-04-13"
    assert result["time"] == "13:45:30"
    assert result["datetime"] == "2023-04-13T13:45:30Z"
  end

  test "parses YAML with flow style collections" do
    yaml = """
    flow_sequence: [1, 2, 3, 4, 5]
    flow_mapping: {key1: value1, key2: value2}
    nested_flow:
      - [a, b, c]
      - {x: 1, y: 2}
    """

    assert {:ok, result} = YamlRustler.parse(yaml)
    assert result["flow_sequence"] == [1, 2, 3, 4, 5]
    assert result["flow_mapping"] == %{"key1" => "value1", "key2" => "value2"}
    assert result["nested_flow"] == [["a", "b", "c"], %{"x" => 1, "y" => 2}]
  end

  test "parses YAML with multiline strings" do
    yaml = """
    literal_block: |
      This is a multiline string.
      It preserves newlines.
      Each line is a separate line.

    folded_string: >
      This is also a multiline string,
      but it will be folded into a
      single line, preserving only
      paragraph breaks.

    literal_with_chomp: |-
      This is a multiline string
      without the trailing newline.
    """

    assert {:ok, result} = YamlRustler.parse(yaml)

    assert result["literal_block"] ==
             "This is a multiline string.\nIt preserves newlines.\nEach line is a separate line.\n"

    assert result["folded_string"] ==
             "This is also a multiline string, but it will be folded into a single line, preserving only paragraph breaks.\n"

    assert result["literal_with_chomp"] ==
             "This is a multiline string\nwithout the trailing newline."
  end

  test "handles empty values correctly" do
    yaml = """
    empty_string: ""
    empty_array: []
    empty_hash: {}
    explicit_null: null
    implicit_null:
    """

    assert {:ok, result} = YamlRustler.parse(yaml)
    assert result["empty_string"] == ""
    assert result["empty_array"] == []
    assert result["empty_hash"] == %{}
    assert result["explicit_null"] == nil
    assert result["implicit_null"] == nil
  end

  test "returns nil for empty input" do
    assert {:ok, nil} == YamlRustler.parse("")
    assert {:ok, nil} == YamlRustler.parse("# only a comment\n")
    assert {:ok, nil} == YamlRustler.parse("---\n")
  end

  test "parses infinity and NaN as atoms" do
    yaml = """
    inf: .inf
    plus_inf: +.Inf
    neg_inf: -.INF
    nan: .NaN
    overflow: 1e400
    """

    assert {:ok, result} = YamlRustler.parse(yaml)
    assert result["inf"] == :infinity
    assert result["plus_inf"] == :infinity
    assert result["neg_inf"] == :neg_infinity
    assert result["nan"] == :nan
    assert result["overflow"] == :infinity
  end

  test "explicit keys take precedence over merged keys regardless of order" do
    yaml = """
    base: &base {x: 1, y: 2, z: 3}
    child:
      y: before
      <<: *base
      z: after
    """

    assert {:ok, result} = YamlRustler.parse(yaml)
    assert result["child"] == %{"x" => 1, "y" => "before", "z" => "after"}
  end

  test "parses non-string keys" do
    yaml = """
    1: int
    true: bool
    ~: null key
    1.5: float
    [1, 2]: list
    """

    assert {:ok, result} = YamlRustler.parse(yaml)

    assert result == %{
             1 => "int",
             true => "bool",
             nil => "null key",
             1.5 => "float",
             [1, 2] => "list"
           }
  end

  test "resolves aliases to scalars and lists" do
    yaml = """
    s: &s hello
    l: &l [1, 2]
    refs: [*s, *l]
    """

    assert {:ok, result} = YamlRustler.parse(yaml)
    assert result["refs"] == ["hello", [1, 2]]
  end

  test "returns an error for an unknown alias" do
    assert {:error, "YAML parsing error: " <> message} = YamlRustler.parse("x: *nope")
    assert message =~ "unknown anchor"
  end

  test "parses integer formats" do
    yaml = """
    hex: 0x1F
    octal: 0o17
    plus: +5
    quoted: "42"
    """

    assert {:ok, %{"hex" => 31, "octal" => 15, "plus" => 5, "quoted" => "42"}} ==
             YamlRustler.parse(yaml)
  end

  test "raises ArgumentError for input that is not valid UTF-8" do
    assert_raise ArgumentError, fn -> YamlRustler.parse(<<0xFF, 0xFE>>) end
  end

  test "parses a large document" do
    yaml =
      "entries:\n" <>
        Enum.map_join(1..5_000, "", fn i ->
          "  - id: #{i}\n    name: \"item #{i}\"\n    tags: [a, b]\n"
        end)

    assert {:ok, %{"entries" => entries}} = YamlRustler.parse(yaml)
    assert length(entries) == 5_000
    assert List.last(entries) == %{"id" => 5_000, "name" => "item 5000", "tags" => ["a", "b"]}
  end
end
