# Review Packet

## Structure

Each review topic lives in its own directory under `review/`:

    review/{NN}-{topic}/
      request.md
      feedback/
        {YYYY-MM-DD}-{seq}-{agent}.md

- `request.md` is the review request written by the implementing agent.
- `feedback/` contains responses from any agent (reviewer or coder).
- A topic may have only feedback and no request (e.g. cross-cutting observations).

## Number Ranges

| Range       | Agent  | Area                |
|-------------|--------|---------------------|
| 1–9999      | coder1 | core scan/build/index |
| 10000–19999 | coder2 | planner integration |
| 20000+      | reserved | future agents     |

Each agent allocates the next number in their range when creating a new topic.

## Feedback Files

Filename: `{YYYY-MM-DD}-{seq}-{agent}.md`

- `seq` orders multiple files from the same day (01, 02, ...).
- `agent` is the short name of the agent that wrote the file.

Each feedback file must include frontmatter:

    ---
    agent: {agent-name}
    role: {coder|reviewer}
    model: {model-id}
    date: {YYYY-MM-DD}
    seq: {N}
    ---

## Agents

| Name     | Role     |
|----------|----------|
| coder1   | coder    |
| coder2   | coder    |
| reviewer | reviewer |

## Rules

- Prefer correctness findings over style comments.
- Focus on behavior, invariants, page/WAL safety, SQL-surface coherence, and missing tests.
- Treat the current on-disk layout as intentional unless a small, concrete defect requires change.
- Do not close review requests yourself. Leave requests open until an outside reviewer has responded.
