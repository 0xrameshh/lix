# lix

Extract and convert AI agent traces into training JSONL. Fast, single binary, no Python required.

## Why

Agent trace files (Claude Code, Codex, Pi, Cursor, etc.) are all different formats. If you want to use them for training, you need to normalize them into something consistent. lix does that — one command, one binary.

There's also [teich](https://github.com/TeichAI/teich) which does this and a lot more. lix is narrower: it just converts traces. The tradeoff is speed (~2-3x faster) and being a single ~8 MB binary with no dependencies.

## Usage

```sh
# Convert a single trace file
lix extract session.jsonl -o training.jsonl

# Convert a directory of traces (auto-detects providers)
lix extract ~/.claude/logs -o claude-data.jsonl

# Clean sensitive data (API keys, paths, etc.)
lix extract trace.jsonl -o clean.jsonl --clean

# Filter by model
lix extract traces/ -o output.jsonl --model claude-4

# Keep incomplete traces (ending mid-conversation)
lix extract trace.jsonl -o output.jsonl --keep-incomplete

# Scan common agent log directories
lix find
```

## Providers

| Provider | Status |
|----------|--------|
| Claude Code (transcript + export) | Supported |
| Codex (session format) | Supported |
| Cursor (native + transcript) | Supported |
| Droid (native) | Supported |
| Hermes (conversation + export) | Supported |
| Pi (native + OpenClaw) | Supported |

## Install

```sh
# Download the binary (once we have releases)
curl -LO https://github.com/0xrameshh/lix/releases/latest/lix-x86_64-unknown-linux-gnu
chmod +x lix

# Or with cargo
cargo install lix
```

## Other commands

```sh
# Inspect a trace file
lix info session.jsonl

# Re-clean an already-extracted JSONL
lix clean input.jsonl -o cleaned.jsonl

# Verify against golden reference
lix verify trace.jsonl --golden expected.jsonl

# Launch web UI
lix studio
```

## Building

```sh
cargo build --release
./target/release/lix --help
```

## License

Apache 2.0
