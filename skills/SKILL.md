# fretka — Web Search Skill

## Purpose

Use `fretka` to search the web when you need up-to-date information, documentation references, or external context that isn't available in the local codebase or your training data.

## Usage

```bash
fretka "<search query>"
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-t, --top-k <N>` | Number of results to return | 5 |
| `-v, --verbose` | Show detailed error messages | off |

### Examples

```bash
# Search for documentation
fretka "python requests retry with backoff"

# Get fewer results for a focused query
fretka -t 3 "typescript extend interface with optional properties"

# Broad research with more results
fretka -t 10 "kubernetes pod graceful shutdown"
```

## Output Format

Results are returned as a numbered markdown list. Each entry contains a linked title and a text snippet:

```
1. [Page Title](https://example.com/page)

   Short description or snippet from the page.

2. [Another Result](https://example.com/other)

   Another snippet of text.
```

## When to Use

- **Unfamiliar APIs or libraries** — search for docs, examples, or changelogs.
- **Error messages** — paste the error string as the query to find known issues or fixes.
- **Best practices** — search for recommended patterns when the right approach isn't obvious.
- **Version-specific information** — find release notes, migration guides, or compatibility details.

## When NOT to Use

- Information already available in the local codebase (use grep/find instead).
- Questions answerable from your own training data with high confidence.
- Queries requiring authenticated or private content (fretka only accesses public web results).

## Tips for Effective Queries

- Be specific: `"reqwest timeout configuration rust"` beats `"http timeout"`.
- Include the language or framework name to narrow results.
- Quote exact error messages for best matches.
- Use `-t 3` for well-defined lookups, `-t 10` when exploring a topic.
