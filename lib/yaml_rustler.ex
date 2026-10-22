defmodule YamlRustler do
  @moduledoc """
  YAML parsing library using Rust's yaml-rust2 via Rustler.
  """

  version = Mix.Project.config()[:version]

  use RustlerPrecompiled,
    otp_app: :yaml_rustler,
    crate: "yamlrustler",
    base_url: "https://github.com/joerichsen/yaml_rustler/releases/download/#{version}",
    force_build: System.get_env("YAML_RUSTLER_BUILD") in ["1", "true"],
    version: version,
    targets: ~w(
      aarch64-apple-darwin
      aarch64-unknown-linux-gnu
      aarch64-unknown-linux-musl
      arm-unknown-linux-gnueabihf
      riscv64gc-unknown-linux-gnu
      x86_64-apple-darwin
      x86_64-pc-windows-gnu
      x86_64-pc-windows-msvc
      x86_64-unknown-linux-gnu
      x86_64-unknown-linux-musl
    )

  # Parsing takes roughly 1 ms per 25 KB. Larger inputs run on a dirty scheduler so
  # they do not block a normal scheduler; smaller ones avoid the dirty scheduler overhead.
  @dirty_threshold 20_000

  @doc """
  Parses a YAML string and returns an Elixir data structure.

  Only the first document is returned, and input without any document
  (an empty string or only comments) returns `{:ok, nil}`. The YAML floats
  `.inf`, `-.inf` and `.nan` cannot be represented on the BEAM and are
  returned as the atoms `:infinity`, `:neg_infinity` and `:nan`.

  ## Options

    * `:ruby` - read plain values the way Ruby's Psych does, for YAML written
      by Ruby (e.g. Rails serialized columns). Psych follows YAML 1.1, where a
      float needs a `.` and an exponent needs a sign, so it writes strings like
      `1e5` or `1.5e3` without quotes. With this option they stay strings
      instead of becoming floats. Defaults to `false` (YAML 1.2).

  ## Examples

      iex> YamlRustler.parse("foo: bar")
      {:ok, %{"foo" => "bar"}}

      iex> YamlRustler.parse("invalid: : yaml")
      {:error, "YAML parsing error: mapping values are not allowed in this context at byte 9 line 1 column 10"}

      iex> YamlRustler.parse("a: 1e5")
      {:ok, %{"a" => 100000.0}}

      iex> YamlRustler.parse("a: 1e5", ruby: true)
      {:ok, %{"a" => "1e5"}}
  """
  @spec parse(binary, keyword) :: {:ok, term} | {:error, binary}
  def parse(yaml_string, opts \\ []) do
    ruby = Keyword.get(opts, :ruby, false)

    if is_binary(yaml_string) and byte_size(yaml_string) > @dirty_threshold do
      parse_dirty_nif(yaml_string, ruby)
    else
      # The normal scheduler NIF leaves documents that need repairing to the dirty one
      case parse_nif(yaml_string, ruby) do
        :reschedule -> parse_dirty_nif(yaml_string, ruby)
        result -> result
      end
    end
  end

  @doc false
  def parse_nif(_yaml_string, _ruby), do: :erlang.nif_error(:nif_not_loaded)

  @doc false
  def parse_dirty_nif(_yaml_string, _ruby), do: :erlang.nif_error(:nif_not_loaded)
end
