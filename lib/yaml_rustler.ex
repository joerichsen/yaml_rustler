defmodule YamlRustler do
  @moduledoc """
  YAML parsing library using Rust's yaml-rust2 via Rustler.
  """

  use Rustler, otp_app: :yaml_rustler, crate: "yamlrustler_native"

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
