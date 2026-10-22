# How to release

## Bump the version

Bump the version in `mix.exs` and `native/yamlrustler/Cargo.toml` (run
`cargo update -p yamlrustler` in `native/yamlrustler` to update Cargo.lock),
for example to 0.1.6. Rename `## Unreleased` in `CHANGELOG.md` to the new
version and date, and update the version in the installation instructions
in `README.md`. Commit and push.

## Tag the release
git tag 0.1.6
git push --tags

## Wait for the release to be built and then regenerate the checksums
The GitHub Action builds a NIF for every target and attaches them to the release.
When all jobs are green, regenerate `checksum-Elixir.YamlRustler.exs` for the new
version (every target in `lib/yaml_rustler.ex` needs an entry):

YAML_RUSTLER_BUILD=1 mix rustler_precompiled.download YamlRustler --all

Commit the updated checksum file.

## Push the release to hex.pm
mix hex.publish

Check the file list it prints: there must be no `priv/native` files in it.


## Useful links
https://orchardlab.dev/posts/dominant-colors/
https://hexdocs.pm/rustler_precompiled/precompilation_guide.html
