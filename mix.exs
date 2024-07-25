defmodule YamlRustler.MixProject do
  use Mix.Project

  def project do
    [
      app: :yaml_rustler,
      version: "0.1.0",
      elixir: "~> 1.16",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      package: package(),
      description: "Fast YAML parsing using Rust's yaml-rust2 via Rustler"
    ]
  end

  defp package do
    [
      name: "yaml_rustler",
      files: ~w(lib priv .formatter.exs mix.exs README* LICENSE*
                native/yamlrustler_native/src native/yamlrustler_native/Cargo.toml),
      licenses: ["MIT"],
      links: %{"GitHub" => "https://github.com/joerichsen/yaml_rustler"}
    ]
  end

  # Run "mix help compile.app" to learn about applications.
  def application do
    [
      extra_applications: [:logger]
    ]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      {:rustler, "~> 0.34.0", runtime: false},
      # For benchmarking
      {:benchee, "~> 1.3", only: :dev},
      # For benchmarking
      {:fast_yaml, "~> 1.0", only: :dev}
    ]
  end
end
