mod dist

set unstable := true

default: install

release:
    just -n release-info
    just -n release-build

release-info:
    just dist::info

release-build:
    just dist::tag
    just dist::build
    just dist::artifacts
    just dist::checksum
    just dist::upload

error:
    cargo run --bin netero -- "hi" > error.txt 2>&1

commit hint="":
    netero commit "{{ hint }}" | git commit -F - --edit

install:
    cargo install --path . --bin netero

install-dev:
    cargo install --path . --bin netero-dev

test-envrc hint="true":
    @direnv allow
    direnv exec . cargo run --bin netero  -- -v  "{{ hint }}"
    @direnv disallow

show-code:
    find src -type f -exec sh -c 'for f; do echo "--- $f ---"; cat "$f"; done' sh {} + | larry "tree -I 'docs|target'" 
