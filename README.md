# Netero

A command-line LLM assistant written in Rust, designed for terminal-centric workflows.

**Also available in Spanish:** [Ver README en español](docs/lang/readme.es.md)

## Project status

Netero is experimental software. Features are incomplete and subject to change.

## Environment variables

Netero requires provider-specific environment variables to be configured.

* `CODE_API_KEY`
  API key used for the `codestral` provider.

* `OPENROUTER_API_KEY`
  API key used for the `openrouter` provider.

The `ollama` provider can be used **locally** without an API key.

At the moment, these are the only supported configuration options. Provider handling is expected to become more flexible in the future.

## Usage

CLI for interacting with language models.

```
Usage: netero [OPTIONS] [PROMPT] [COMMAND]
```

If input is provided via `stdin`, it will be used as additional context for the prompt.

### Commands

* `chat`
  Open a minimal chat session

* `commit`
  Generate a commit message from staged changes

* `prompt`
  Send a prompt to the language model and print the response

### Arguments

* `[PROMPT]`
  Prompt passed to the language model

### Options

* `-p, --provider <PROVIDER>`
  Language model provider (default: `codestral`)

* `-v, --verbose`
  Enable verbose output

* `-h, --help`
  Print help

* `-V, --version`
  Print version

## Examples

### Basic prompt

```sh
netero "Explain the difference between hard links and symbolic links"
```

### Using stdin for longer prompts

```sh
cat README.md | netero "Summarize the project README"
```

### Generate a Git commit message

```sh
netero commit | git commit -F - --edit
```

### Using a different provider

```sh
netero -p openrouter "Explain how systemd manages services"
```

### Verbose output

```sh
netero -v "Explain the Rust ownership model"
```

### Process a man page

```sh
man tmux | netero "How can I split a tmux window?"
```

### Analyze command output

```sh
ps aux | netero "Which processes are consuming the most resources?"
```

### Pipe output to another command
```sh
ss -tulpen | netero "Summarize active listening sockets" | mdless
```

## License

BSD 2-Clause
