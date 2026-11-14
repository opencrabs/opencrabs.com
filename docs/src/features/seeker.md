# Seeker

Seeker is one field over everything the app can do. Press `alt+space` in the terminal or in the window (`cmd+k` opens it too on a Mac), type what you want, and the card ranks the answer: a conversation, a command, a screen, a skill, a channel, a model, a docs page, or a new session built from the words you just typed.

It replaced the old command palette, and the difference is the point. A palette lists commands and hopes you remember the name. Seeker also looks at your own history and at the documentation, so "that thing about telegram polls" can land on the right page even when you do not know what the page is called.

## What it finds

| Kind | What a row is |
|---|---|
| Conversation | A session from the rail, opened on Enter |
| Command | A compiled slash command, run on Enter |
| Surface | A screen or panel the app can show |
| Action | A direct interface action, such as opening or closing the sidebar |
| Help | A chord from the shared help table, so the card teaches the keys it answers |
| Model | A model from the provider catalogue |
| Skill | A skill from `skills/`, described in its own words |
| User command | A command from your own `commands.toml` |
| Channel | A messaging channel the agent can be reached on. Picking the row starts its setup |
| Docs | A page on docs.opencrabs.com, opened in your browser |
| New session | The row you get when nothing matched: it starts a fresh session with your typed text as the request |

## How matching works

Whole words, not loose letter sequences. Each word you type has to land on a real word in the row, so a query cannot drag in an unrelated row by matching one letter out of every field. The fields are weighted, and the ranking follows the weight: a command's own name outranks a title, a title outranks keywords, and keywords outrank a subtitle.

Two things bend that rule on purpose. A skill is matched against its description as prose, which is how "beat synced film" finds a skill whose title says nothing about it. A session is matched against its title with substring tolerance, because a session name is not always a dictionary word: `proxy` still finds `OCProxy`.

If a query matches nothing, the card does not just sit there. It offers to start a new session with what you typed, which is usually what you wanted anyway.

## The docs index

The documentation rows are not a hand-picked sample. The index is the site's own table of contents, generated from the published sitemap by `scripts/gen-docs-index.py`, and a test pins the coverage: a page that ships without a row fails the suite instead of quietly missing from the card.

The case studies are deliberately left out. With them indexed, the word `telegram` armed a story above the setup page that answers the question, and a search that lands you on an anecdote is worse than no result.

## Related

- [Desktop app](./desktop-app.md) for the window's own chords
- [Sessions](./sessions.md) for the rail the conversation rows come from
- [Skills System](./skills.md) for what the skill rows point at
