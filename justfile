set fallback := true
set unstable := true

default: test-ollama

commit hint="":
  netero commit "{{ hint }}" | git commit -F - --edit

install:
    cargo install --path . --bin netero-dev

sync:
    git switch default
    git merge development
    git switch development
    git push --all

test-ollama:
    RUSTFLAGS="-Awarnings" cargo run --bin netero-dev --quiet -- -v -p ollama  "escribe un poema"

test-chat:
    RUSTFLAGS="-Awarnings" cargo run --bin netero-dev --quiet -- -v -p ollama  chat "work"

show:
    find src -type f -exec sh -c 'for f; do echo "--- $f ---"; cat "$f"; done' sh {} + | larry "tree -I 'docs|target'" 



