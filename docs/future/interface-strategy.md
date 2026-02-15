# Future Interface Strategy: How Quip Should Use Timely

## Goal
Use Timely as a reliable activity-intelligence layer so I can answer questions fast ("what did I do in the last 30m?") and proactively provide useful summaries.

## Core Interface Principles

1. **CLI-first, JSON-first**
   - Prefer Timely CLI commands with `--json` output.
   - Avoid parsing human-readable text when structured output exists.

2. **Daemon-required workflow**
   - Timely must be running continuously for accurate history.
   - First check in any reporting flow: `timely daemon status`.

3. **Small set of canonical queries**
   - Current context: `timely now --json`
   - Short-window summary: `timely summary --from 30m --to now --by app --json`
   - Timeline inspection: `timely timeline --from 2h --to now --json`
   - Daily recap: `timely summary --from today --to now --by category --json`

4. **Deterministic time ranges**
   - Use supported ranges (`30m`, `2h`, `today`, `yesterday`, etc.).
   - Normalize user requests like "last half hour" to `30m`.

## Operational Flow (per request)

1. Validate service:
   - `timely daemon status`
2. Run the narrowest useful query (usually summary + optional timeline).
3. Produce concise output:
   - top apps/categories
   - total tracked duration
   - notable switches/interruptions
4. If no data:
   - clearly say why (daemon off / no events in window)
   - provide next action (`timely daemon start`)

## Quality Targets

- **Fast response**: first-pass answer from summary command only.
- **Accuracy**: avoid inferred data when Timely reports no data.
- **Explainability**: include exact queried range in replies.

## Future Integrations

1. **Auto-reports**
   - Scheduled snapshots (midday + end-of-day).
   - Weekly category trend summary.

2. **Focus analytics**
   - Detect context switching bursts.
   - Highlight uninterrupted deep-work blocks.

3. **Messaging delivery**
   - Send summary cards to Telegram/WhatsApp on request.

4. **Rule-assisted categorization**
   - Expand `timely categorize` rules for cleaner category-level reporting.

## Command Snippets

```bash
# Health check
timely daemon status

# Last 30 minutes by app
timely summary --from 30m --to now --by app --json

# Last 2 hours timeline
timely timeline --from 2h --to now --json

# Today by category
timely summary --from today --to now --by category --json
```

## UX Preference for Assistant Replies

For quick asks, respond in this format:
- **Range:** last 30m
- **Top apps:** App A (12m), App B (9m), App C (6m)
- **Total tracked:** 27m
- **Pattern:** moderate switching (5 context changes)
- **Actionable nudge:** suggest focus block or next task

This keeps insights short, useful, and repeatable.
