defmodule YamlRustler.MixProject do
  use Mix.Project

  def project do
    [
      app: :yaml_rustler,
      version: "0.1.5",
      elixir: "~> 1.16",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      package: package(),
      description: "Fast YAML parsing using Rust's yaml-rust2 via Rustler",
      name: "YamlRustler",
      source_url: "https://github.com/joerichsen/yaml_rustler",
      homepage_url: "https://github.com/joerichsen/yaml_rustler",
      docs: docs()
    ]
  end

  defp package do
    [
      name: "yaml_rustler",
      files: ~w(lib priv .formatter.exs mix.exs README* LICENSE*
                native/yamlrustler/src native/yamlrustler/Cargo.toml
                native/yamlrustler/.cargo/config.toml checksum-*.exs),
      licenses: ["MIT"],
      links: %{"GitHub" => "https://github.com/joerichsen/yaml_rustler"}
    ]
  end

  defp docs do
    [
      # This will display the README on the main page
      main: "readme",
      extras: ["README.md"]
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
      {:rustler_precompiled, "~> 0.8.2", runtime: false},
      {:ex_doc, ">= 0.0.0", only: :dev, runtime: false},
      # For benchmarking
      {:benchee, "~> 1.3", only: :dev},
      # For benchmarking
      {:fast_yaml, "~> 1.0", only: :dev}
    ]
  end
end
