# fretka

A web-search CLI for coding agents. Searches DuckDuckGo and returns cleanly formatted results.

## Install

```bash
cargo install fretka
```

### Add the skill to your coding tools

Register fretka as a skill so your coding agent knows when and how to search the web:

```bash
fretka install-skill
```

The wizard detects your installed tools — Claude Code, Codex, Gemini CLI, Cline, OpenCode, GitHub Copilot, and Mistral Vibe — and copies the skill file into each one you select.

## Usage

```bash
fretka "python requests retry with backoff"
fretka -t 3 "typescript extend interface"
fretka --fetch "kubernetes pod graceful shutdown"
fretka -f json "rust serde deserialize enum"
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-t, --top-k <N>` | Number of results to return (1–100) | 5 |
| `--fetch` | Fetch each URL and extract readable content | off |
| `-f, --format <FMT>` | Output format: `markdown` or `json` | markdown |
| `-v, --verbose` | Show detailed error messages | off |

## License

MIT
