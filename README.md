# Introduction

Netero 🫶 is a CLI for LLMs, oriented toward advanced GNU/Linux users.
It runs from the command line, integrates into pipelines,
and offers a minimal chat with command expansion.

The more skilled you are with shells, the more you will get out of this tool.

**Also in Spanish:** [View README in Spanish](./docs/es/)

---

## Environment Variables

It is configured via environment variables.

### Default provider (`codestral`)

* `CODE_API_KEY`
  API key for the default provider.

### Custom provider (OpenAI-compatible)

* `NETERO_URL`
  Chat completions endpoint URL.

* `NETERO_MODEL`
  Name of the selected model.

* `NETERO_API_KEY`
  Optional API key for the custom provider.

---

## Usage

```
Usage: netero [OPTIONS] [PROMPT] [COMMAND]
```

If there is input from `stdin`, it is added as additional context.

---

### Commands

* `chat`
  Opens a minimal chat session.

* `commit`
  Generates a commit message from staged changes.

* `completion`
  Generates shell autocompletion scripts.

* `prompt`
  Sends a prompt to the model and displays the response.

---

### Arguments

* `[PROMPT]`
  Prompt sent to the model.

---

### Options

* `-v, --verbose`
  Enables verbose output.

* `-t, --trace`
  Shows the prompts sent and the responses received (debug mode).

* `-h, --help`
  Displays help.

* `-V, --version`
  Displays the version.

You can use the `--trace` option to inspect the prompts being sent:

```
netero --trace
```

---

## Interactive Chat

### Built-in Commands

* `/help`
  Displays help.

* `/clean`
  Clears chat history.

* `/add`
  Attaches files to the context.

* `/trans`
  Translates text.

* `/eval`
  Evaluates an arithmetic expression.

* `/save`
  Saves the user request to a file.

* `/stream`
  Enables or disables streaming mode in the model response.

---

### Inline Command Execution

You can execute any valid shell command using:

```
#!(...)
```

The command output will be attached to the next prompt sent to the model.

Example:

```sh
➜ #!(sudo nmap scanme.nmap.org) analyze
```

The generated prompt would be similar to:

```text
:: END CHAT HISTORY (SYSTEM) ::
:: COMMAND OUTPUT (SYSTEM) ::
[section]
[command]
sudo nmap scanme.nmap.org

[stdout]
...
[end section]
:: END COMMAND OUTPUT (SYSTEM) ::
:: USER MESSAGE ::
analyze
:: END USER MESSAGE ::
:: RESPONSE ::
The command `sudo nmap scanme.nmap.org` completed a successful scan of the host `scanme.nmap.org` (IPv4: 45.33.32.156, IPv6: 2600:3c01::f03c:91ff:fe18:bb2f). Four open ports and one filtered port were detected:

- **Open ports**:
  - 22/tcp (SSH)
  - 80/tcp (HTTP)
  - 9929/tcp (nping-echo)
  - 31337/tcp (Elite)

- **Filtered port**:
  - 646/tcp (LDP)

The scan took 4.89 seconds. No critical errors were found.
```

⚠ Commands are executed in the user's shell without sandboxing.

---

## Using Paths in Prompts

Netero automatically detects absolute and relative paths in the prompt.

Any word that:

* Starts with `/` (absolute path)
* Starts with `./` or `../` (relative path)
* And corresponds to an **existing file**

will be interpreted as a valid path and its contents will be automatically attached to the context sent to the model.

No special commands are required.

---

### In `chat` Mode

Within an interactive session:

```sh
netero chat
```

You can write:

```
Review this file and suggest improvements: ./src/main.rs
```

If `./src/main.rs` exists and is a file, its contents will be automatically attached to the prompt.

---

### In Direct Prompt Mode

It also works from the command line:

```sh
netero "Translate this file ./README.en.md"
```

If `./README.en.md` exists, its contents will be automatically included in the context before sending the request to the model.

---

### Behavior

* Only existing files are attached.
* Directories are not attached.
* Multiple paths can be included in the same prompt.
* Paths are resolved from the current directory.

---

## Examples

### 1. Direct prompt

```sh
netero "Explain how io_uring works in Linux"
```

---

### 2. Prompt without quotes

```sh
netero Explain the ownership model in Rust
```

---

### 3. Using `stdin` as context

```sh
cat Cargo.toml | netero "Describe the main dependencies"
```

---

### 4. Analyzing command output

```sh
dmesg | tail -n 50 | netero "Are there any relevant errors in these logs?"
```

---

### 5. Generate commit message from staging

```sh
netero commit | git commit -F -
```

---

### 6. Generate message with custom convention

```sh
netero commit -c .repo/convencion.txt
```

---

### 7. Verbose mode

```sh
netero -v "Explain what a mutex is in Rust"
```

---

### 8. Inspect sent prompts (debug mode)

Netero runs a debug socket that listens to all prompts sent by running instances.

```sh
netero --trace
```

---

### 9. Use custom provider

```sh
export NETERO_URL="https://api.example.com/v1/chat/completions"
export NETERO_MODEL="my-model"
export NETERO_API_KEY="your-api-key"

netero "Describe the CFS scheduling algorithm"
```

---

### 10. Generate shell autocompletion

```sh
netero completion bash
```

---

### 11. Start interactive session

```sh
netero chat
```

---

### 12. Execute command inside chat

Within the session:

```
Analyze this output:
#!(free -h)
```

---

## License

BSD 2-Clause
