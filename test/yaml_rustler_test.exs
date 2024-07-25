defmodule YamlRustlerTest do
  use ExUnit.Case
  doctest YamlRustler

  test "greets the world" do
    assert YamlRustler.hello() == :world
  end
end
