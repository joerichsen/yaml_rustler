defmodule YamlRustler do
  @moduledoc """
  YAML parsing library using Rust's yaml-rust2 via Rustler.
  """

  alias YamlRustler.Native

  @doc """
  Parses a YAML string and returns an Elixir data structure.

  ## Examples

      iex> YamlRustler.parse("foo: bar")
      {:ok, %{"foo" => "bar"}}

      iex> YamlRustler.parse("invalid: : yaml")
      {:error, "YAML parsing error: ..."}
  """
  @spec parse(String.t()) :: {:ok, term()} | {:error, String.t()}
  def parse(yaml_string) do
    case Native.parse(yaml_string) do
      {:error, reason} -> {:error, reason}
      result -> {:ok, result}
    end
  end
end
