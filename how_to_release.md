# How to release

## Bump the version in `mix.exs`

Bump the version in `mix.exs` for example to 0.1.6

## Tag the release
git tag 0.1.6
git push --tags

## Wait for the release to be built and then run this
YAML_RUSTLER_BUILD=1 mix rustler_precompiled.download YamlRustler --all

## Push the release to hex.pm
mix hex.publish


## Useful links
https://orchardlab.dev/posts/dominant-colors/
https://hexdocs.pm/rustler_precompiled/precompilation_guide.html