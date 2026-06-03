# llmkeys

**English** | [简体中文](./README.zh-CN.md)

> A keys-and-config manager for multiple LLM providers. Pull an API key out of your
> keychain / Bitwarden and generate a paste-ready `.env` snippet or LangChain code
> snippet in one command — no more googling the provider, digging through docs for the
> `base_url`, then hunting for the key in some dashboard every single time.

**Stand on the shoulders of giants, don't reinvent the wheel**: the provider/model
catalog reuses [models.dev](https://models.dev), key storage reuses your OS keychain and
your existing Bitwarden/Vaultwarden — llmkeys is only the thin integration layer on top.

> ✅ **Current status: v1 shipped (macOS only).**

---

## What it solves

Day-to-day development means constantly switching providers/models to compare and test.
Three pain points:

1. **Re-googling dead knowledge** — every provider's `base_url`, model-ID naming, and
   embedding model name has to be looked up again each time.
2. **Scattered, insecure keys** — API keys jotted into plaintext notes risk leaking and
   getting lost.
3. **Tedious switching** — using a given model means hand-assembling a whole config.

llmkeys collapses all of that into a few commands: list providers, view config, pull a
key and assemble a `.env` or code snippet.

## Core principle: secrets and config live apart

| Category | Contents | Stored in | On disk? |
|---|---|---|---|
| Secret | API key | keychain / `bw` | No (config stores only a **reference**) |
| Non-secret config | base_url, model IDs, embedding, env var names | config dir (snapshot + models.dev + overrides) | Yes (plaintext, committable/shareable) |

llmkeys itself **never holds or persists** any secret; a key, once pulled, is wrapped in
`Zeroizing` the whole way and wiped the moment it leaves scope.

## Three concepts you must know

**① Credential reference URI** (config stores only the reference, never the key):

```
<backend>:<locator>[#profile]
keychain:openrouter            # macOS keychain, default account
keychain:openrouter#work       # use #profile to separate multiple accounts
bw:item/OpenRouter API Key     # Bitwarden, by item name
bw:id/2a16-445b-...            # Bitwarden, by item id (survives renames, more stable)
env:OPENROUTER_API_KEY         # environment-variable fallback
```

**② keychain layout**: `service = "dev.mars.llmkeys"`, `account = "<provider>[#profile]"`,
one entry per key.

**③ Three-layer config merge** (low → high): **built-in snapshot < models.dev cache < your
overrides** (`~/.config/llmkeys/providers.toml`). **Field-level merge, your write always
wins** — override only the fields you want, the rest fall through to the lower layer.

> Note: Bitwarden always goes through **`bw` (Password Manager CLI)**, which works with
> self-hosted Vaultwarden; it does **not** use `bws` (Secrets Manager) — that one is
> non-open-source and unsupported by Vaultwarden.

---

## Install

macOS only (v1). **Homebrew is recommended** — a prebuilt binary, no Rust toolchain needed;
brew strips the quarantine attribute automatically, so Gatekeeper won't block it.

### Recommended: Homebrew

```sh
brew install csthink/tap/llmkeys
llmkeys --version        # verify
```

> Equivalent to `brew tap csthink/tap && brew install llmkeys`; upgrade later with
> `brew upgrade llmkeys`.

### Alternative: build from source

Requires the **Rust toolchain** (`rustup`, stable). If you don't have it:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh   # official one-liner to install rustup (incl. cargo)
source "$HOME/.cargo/env"                                        # make it effective in the current shell
cargo --version                                                  # verify: should print a version
```

> Still getting `cargo: command not found` after installing? Most likely the terminal was
> already open before the install — `source "$HOME/.cargo/env"` or open a new terminal.

Install as a single binary (lands at `~/.cargo/bin/llmkeys`):

```sh
git clone https://github.com/csthink/llmkeys.git
cd llmkeys
cargo install --path .
llmkeys --help        # verify
```

> Just want a local build without installing: `cargo build --release`, artifact at
> `target/release/llmkeys`.

### Optional: Bitwarden CLI (only if you pull keys via the `bw` backend)

[Bitwarden CLI `bw`](https://bitwarden.com/help/cli/):

```sh
brew install bitwarden-cli      # or: npm install -g @bitwarden/cli
bw --version                    # verify (note it's bw, not bws)
```

### Uninstall / switch install method

The two methods land in different places (brew → `/opt/homebrew/bin`, source →
`~/.cargo/bin`) and **don't overwrite each other**; with both installed, PATH order decides
which `llmkeys` wins (`type -a llmkeys` shows them all). Uninstall the old one before
switching to avoid confusion.

**Uninstall the Homebrew build:**

```sh
brew uninstall llmkeys
brew untap csthink/tap      # optional: drop the tap if you no longer want updates
```

**Uninstall the source (cargo) build:**

```sh
cargo uninstall llmkeys     # removes ~/.cargo/bin/llmkeys
```

**Switch from source to Homebrew** (common case):

```sh
cargo uninstall llmkeys                 # uninstall the source build first
brew install csthink/tap/llmkeys        # then install the brew build
```

> Uninstalling only touches the binary. Your **config** (`~/.config/llmkeys/`),
> **models.dev cache** (`~/.cache/llmkeys/`), and **keychain keys**
> (`service = dev.mars.llmkeys`) are untouched — switching methods loses nothing. To wipe
> config/cache entirely: `rm -rf ~/.config/llmkeys ~/.cache/llmkeys`; for keychain keys use
> [`security delete`](#managing-keys-in-the-keychain).

---

## Walkthrough: the keychain route (default)

The simplest happy path, from viewing a provider to pulling a `.env`, never touching
Bitwarden.

### 1. List all providers

The merged three-layer list (openrouter / siliconflow / aliyun_bailian / deepseek come
pre-seeded):

```sh
llmkeys list
```

### 2. View a provider's full config

The key is shown only as a **reference**, never in plaintext:

```sh
llmkeys show openrouter
```

```text
openrouter
  display_name    : OpenRouter
  base_url        : https://openrouter.ai/api/v1
  key_ref         : keychain:openrouter   (reference, plaintext not shown)
  env_prefix      : OPENROUTER
  models.chat     : openai/gpt-5.5
  models.embedding: baai/bge-m3
```

### 3. Store an API key (into the keychain)

Interactive paste — **never via argv, never into shell history, input not echoed**:

```sh
llmkeys key set openrouter
# Paste the API key for keychain:openrouter (input hidden, press Enter to confirm):
```

### 4. Check whether it was stored

Returns only `yes` / `no`, never the plaintext:

```sh
llmkeys key check openrouter      # → yes
```

### 5. Pull the config into a `.env` snippet

The key is pulled from the keychain, ready to paste straight into your project's `.env`:

```sh
llmkeys env openrouter
```

```text
OPENROUTER_API_KEY=sk-...
OPENROUTER_BASE_URL=https://openrouter.ai/api/v1
OPENROUTER_MODEL=openai/gpt-5.5
OPENROUTER_EMBEDDING_MODEL=baai/bge-m3
```

> Var name = `env_prefix` + a fixed suffix (`_API_KEY` / `_BASE_URL` / `_MODEL` /
> `_EMBEDDING_MODEL`); a missing model role drops its line (e.g. deepseek has no embedding,
> so no `_EMBEDDING_MODEL` line).

Send to the clipboard instead of printing:

```sh
llmkeys env openrouter --copy
```

### 6. Or assemble a LangChain code snippet

```sh
llmkeys code openrouter
```

```python
from langchain_openai import ChatOpenAI
llm = ChatOpenAI(
    base_url="https://openrouter.ai/api/v1",
    model="openai/gpt-5.5",
    api_key="sk-...",
)
```

---

## Switch to pulling keys from Bitwarden / Vaultwarden

Put a provider's key in your existing Bitwarden vault (self-hosted Vaultwarden works too)
and llmkeys fetches it on demand via `bw`.

### 1. Create the item in Bitwarden

It must be a **Login** type (llmkeys calls `bw get password`, and only Login has a password
field): name it anything (e.g. `DEEPSEEK_API_KEY`), put the API key into the **password
field**, leave the rest empty, then sync.

### 2. Log in and unlock the bw CLI

llmkeys calls `bw` **non-interactively**, so you must unlock and export a session in your
shell first:

```sh
bw config server https://your-vaultwarden.example.com   # only for self-hosted; skip for the official cloud
bw login                                                # first-time login
export BW_SESSION="$(bw unlock --raw)"                  # unlock and export the session (every new terminal)
bw sync                                                 # pull the latest items
```

### 3. Get the item id (more stable than referencing by name)

Search by the name you stored to get the id; renames won't break it:

```sh
bw list items --search DEEPSEEK_API_KEY     # take the "id" field from the output
```

### 4. Point key_ref at bw (in the overrides file)

Edit `~/.config/llmkeys/providers.toml`:

```toml
[providers.deepseek]
key_ref = "bw:id/86f99441-86ee-4632-8fda-21816a94fce2"   # use the id from the previous step
# or by item name: key_ref = "bw:item/DEEPSEEK_API_KEY"
```

> The locator **only supports** `item/<name>` or `id/<id>`; any other prefix (e.g.
> `bw:llm/...`) errors out.

### 5. End-to-end verification

```sh
llmkeys show deepseek      # key_ref should show bw:id/... (reference, plaintext not shown)
llmkeys env deepseek
```

```text
DEEPSEEK_API_KEY=sk-...
DEEPSEEK_BASE_URL=https://api.deepseek.com/v1
DEEPSEEK_MODEL=deepseek-v4-pro
```

### bw failure messages at a glance

Every failure is a human-readable message (no panic, no plaintext key):

| Scenario | How to trigger | llmkeys message |
|---|---|---|
| Locked | pull a key without `export BW_SESSION` | `Bitwarden is locked: run bw unlock first …` |
| Not logged in | pull a key after `bw logout` | `Not logged in to Bitwarden: run bw login first …` |
| Item not found | `key_ref` points at a non-existent name/id | `No matching item found in Bitwarden` |
| Item has no password | item isn't a Login / password field empty | `bw returned an empty password: the item may have no password field …` |
| Bad locator | e.g. `bw:llm/deepseek` | `Unknown bw locator type llm: only item / id are supported` |

> `llmkeys key set/check` **only deal with the keychain**, never writing or checking bw (v1
> does not write to bw through llmkeys); verify bw values directly with `llmkeys env` /
> `llmkeys code`.

---

## Multiple accounts (`#profile`)

Multiple key sets for one provider (personal / work) are separated by `#profile`:

```sh
llmkeys key set openrouter#work          # store the work-account key
llmkeys key check openrouter#work
llmkeys env openrouter --profile work    # pull the work account
```

In the keychain this maps to `account = "openrouter#work"`; a bw reference is written
`bw:item/...#work`.

---

## Customize / fill in providers (local overrides)

Write overrides in `~/.config/llmkeys/providers.toml` (three-layer merge, field-level, your
write always wins; **store only non-secret config, never the key**):

```toml
# change only base_url (e.g. route through your own proxy), other fields stay from the snapshot
[providers.openrouter]
base_url = "https://my-proxy.local/v1"

# add a new provider
[providers.mycorp]
display_name = "MyCorp"
base_url     = "https://api.mycorp.com/v1"
key_ref      = "keychain:mycorp"
env_prefix   = "MYCORP"
  [providers.mycorp.models]
  chat      = "mycorp-large"
  embedding = "mycorp-embed"
```

Refresh the models.dev cache (keeps the old cache on failure):

```sh
llmkeys refresh
```

> Domestic-China providers (SiliconFlow / Aliyun Bailian) are incompletely covered by
> models.dev — rely on the **snapshot / your overrides**, don't wait on upstream.

---

## Managing keys in the keychain

llmkeys only has `key set` / `key check` and **no list/delete subcommands** — use macOS's
built-in `security` (entries are fixed at `service = "dev.mars.llmkeys"`,
`account = "<provider>[#profile]"`).

**List which keys llmkeys has stored** (read-only metadata, no plaintext, no password
prompt):

```sh
security dump-keychain 2>/dev/null | awk '
  /"acct"<blob>=/ { a=$0; sub(/.*"acct"<blob>="/,"",a); sub(/".*/,"",a); acct=a }
  /"svce"<blob>=/ { s=$0; sub(/.*"svce"<blob>="/,"",s); sub(/".*/,"",s);
                    if (s=="dev.mars.llmkeys") print acct }'
```

**Check whether a single one exists** (exit code 0 = exists):

```sh
security find-generic-password -s "dev.mars.llmkeys" -a "openrouter" >/dev/null 2>&1 && echo yes || echo no
```

**Delete a key** (irreversible; confirm you have a backup elsewhere first):

```sh
security delete-generic-password -s "dev.mars.llmkeys" -a "openrouter"        # default profile
security delete-generic-password -s "dev.mars.llmkeys" -a "openrouter#work"   # a specific profile
```

> **Migrate a provider wholesale from keychain to bw**: ① create the item in Bitwarden →
> ② change `key_ref` to `bw:id/<id>` → ③ verify with `llmkeys env <id>` that it pulls →
> ④ **only after confirming** delete the old keychain entry with the `security delete` above.
> Don't reverse the order, or you may lose the key.

---

## Command reference

| Command | What it does |
|---|---|
| `llmkeys list` | List all merged providers (name + base_url) |
| `llmkeys show <id>` | Show a provider's config (key shown as a reference, no plaintext) |
| `llmkeys key set <id[#profile]>` | Interactively paste a key, write it into the **keychain** |
| `llmkeys key check <id[#profile]>` | Check whether the keychain has that key (yes/no) |
| `llmkeys env <id> [--profile p] [--copy]` | Output a `.env` snippet |
| `llmkeys code <id> [--profile p] [--copy]` | Output a LangChain (`ChatOpenAI`) snippet |
| `llmkeys refresh` | Re-fetch the models.dev cache (keeps the old cache on failure) |

> Delete keychain keys with the system `security` (see
> [Managing keys in the keychain](#managing-keys-in-the-keychain)); llmkeys provides no
> delete subcommand.

---

## Stack & scope (v1)

- Language: Rust (single static binary); platform: **macOS only**.
- Key backends: `keychain` (default) / `bw` / `env`.
- Catalog: models.dev fetch + built-in snapshot fallback + user local overrides.
- Model roles: `chat` + `embedding` (schema reserved for extension).
- Output: `.env` snippet + LangChain code snippet.

**Not in v1** (the data model reserves room for them): secret injection into a subprocess
(`run --`), Linux/headless, Vault backend, GUI, code signing/notarization.

## Design docs

The full spec lives in [`docs/`](./docs/):

| Doc | Contents |
|---|---|
| [proposal.md](./docs/proposal.md) | Motivation, scope, locked decisions |
| [spec.md](./docs/spec.md) | Testable behavior contract (commands, reference syntax, schema, output formats) |
| [design.md](./docs/design.md) | Rust architecture, crate structure, the SecretStore trait |
| [tasks.md](./docs/tasks.md) | T0–T7 implementation breakdown (one owner per file) |
| [workflow.md](./docs/workflow.md) | Minimal dev process: cadence, the three red lines, review on demand |

Built-in provider snapshot:
[`snapshot/providers.snapshot.toml`](./snapshot/providers.snapshot.toml) (runtime resource).

> The design docs are written in Chinese; see [`README.zh-CN.md`](./README.zh-CN.md) for the
> Chinese version of this README.

## License

[MIT](./LICENSE) © 2026 mars

Open source, no revenue goal — built to free developers from repetitive busywork.
