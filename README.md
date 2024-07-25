# YamlRustler

YamlRustler is a fast YAML parsing library for Elixir, leveraging the power of Rust's [yaml-rust2](https://github.com/Ethiraric/yaml-rust2) library via [Rustler](https://github.com/rusterlium/rustler).

## Installation

Add `yaml_rustler` to your list of dependencies in `mix.exs`:

```elixir
def deps do
  [
    {:yaml_rustler, "~> 0.1.0"}
  ]
end
```

## Usage

```elixir
iex> yaml = """
...> foo: bar
...> baz:
...>   - qux
...>   - quux
...> """
iex> YamlRustler.parse(yaml)
{:ok, %{"foo" => "bar", "baz" => ["qux", "quux"]}}
```

## Performance

YamlRustler has been benchmarked against fast_yaml, another popular YAML parsing library for Elixir. Here are the results:

```
Name                   ips        average  deviation         median         99th %
yaml_rustler       49.00 K       20.41 μs   ±126.22%       18.92 μs       41.46 μs
fast_yaml          46.28 K       21.61 μs    ±99.17%       20.25 μs       37.38 μs

Comparison:
yaml_rustler       49.00 K
fast_yaml          46.28 K - 1.06x slower +1.20 μs
```

These benchmarks show that YamlRustler is slightly faster than fast_yaml, performing about 1.06x better on average.

## Features

- Fast YAML parsing using Rust's yaml-rust2 library
- Fully compatible with Elixir data structures
- Supports parsing of complex YAML structures

## Limitations

- Currently only supports parsing a single YAML document per string. Multi-document YAML files are not supported yet.
- Does not support YAML dumping (converting Elixir structures to YAML) at this time.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
