set fallback := true
set unstable := true

default: test

error:
  cargo run --bin netero -- "hi" > error.txt 2>&1

commit hint="":
  netero commit "{{ hint }}" | git commit -F - --edit

chat:
    cargo run --bin netero -- chat

install:
    cargo install --path . --bin netero

install-dev:
    cargo install --path . --bin netero-dev

sync:
    git switch default
    git merge development
    git switch development
    git push --all

test:
    man yes | cargo run --bin netero-dev --quiet -- -v chat 

test-envrc hint="¿Cual es tu nombre?":
    direnv allow
    direnv exec . cargo run --bin netero-dev --quiet -- -v "{{ hint }}"
    direnv disallow

show:
    find src -type f -exec sh -c 'for f; do echo "--- $f ---"; cat "$f"; done' sh {} + | larry "tree -I 'docs|target'" 



