defmodule YamlRustler do
  @moduledoc """
  YAML parsing library using Rust's yaml-rust2 via Rustler.
  """

  version = Mix.Project.config()[:version]

  use RustlerPrecompiled,
    otp_app: :yaml_rustler,
    crate: "yamlrustler_native",
    base_url: "https://github.com/joerichsen/yaml_rustler/releases/download/#{version}",
    force_build: System.get_env("YAML_RUSTLER_BUILD") in ["1", "true"],
    version: version,
    targets: ~w(
      aarch64-apple-darwin
      aarch64-unknown-linux-gnu
      arm-unknown-linux-gnueabihf
      riscv64gc-unknown-linux-gnu
      x86_64-apple-darwin
      x86_64-pc-windows-gnu
      x86_64-pc-windows-msvc
      x86_64-unknown-linux-gnu
      x86_64-unknown-linux-musl
    )

  @doc """
  Parses a YAML string and returns an Elixir data structure.

  ## Examples

      iex> YamlRustler.parse("foo: bar")
      {:ok, %{"foo" => "bar"}}

      iex> YamlRustler.parse("invalid: : yaml")
      {:error, "YAML parsing error: mapping values are not allowed in this context at byte 9 line 1 column 10"}
  """
  @spec parse(binary) :: {:ok, term} | {:error, binary}
  def parse(_yaml_string), do: :erlang.nif_error(:nif_not_loaded)
end
