# Message Actions

Every row in the window carries its own actions. Hover a message and the affordance appears in its corner; right-click opens the menu. Nothing is global, so acting on one answer never depends on which row the caret happens to be in.

## On a single row

| Action | Where it is | What it does |
|---|---|---|
| Edit | The pencil on your own message, on hover | Opens the row in the composer with its text, so you fix a typo instead of sending a correction |
| Copy as Markdown | The copy glyph on an answer, on hover | Puts the stored markdown source on the clipboard, headings and fences intact. The glyph becomes a check in the same box, so the confirmation costs the row no height |
| React | The row menu, then the crab pack | Puts one of the pack's pictures on the row. Picking the same one again takes it back |
| Reply | The row menu on an answer | Quotes the picked rows into the composer, so your next message carries the context instead of pointing at it |
| Forward | The row menu on an answer | Opens the destination picker: the rows going out, a query field, and the sessions a forward can land in. The forward is staged for review, not sent on the click |
| Select | The row menu | Enters select mode and arms the bulk bar |

The menu is not identical on every row. `Reply` and `Forward` belong to answers, because quoting an answer back or sending it on is what a reader does with it. Your own rows keep the shorter menu: a press on a message you already sent should not offer to reply to it or send it somewhere else.

A turn that is still streaming carries the same actions on the exact live entry, so you can copy an answer or react to it while it is being written instead of waiting for the turn to land.

## On a selection

Select mode replaces the composer with a bar that shows how many rows are picked, and it carries the bulk actions: Copy, Reply, Forward and Cancel.

Picks are keyed by row, not by position. A row that scrolls out of the viewport, or a refresh that re-renders only the visible window, does not lose what you already picked. A shift-click unions the rows between the anchor and the one you clicked, so a second run of rows can be added without dropping the first. Forward and Reply both read the picks in transcript order, which means the destination reads the same conversation you selected rather than the order you happened to click.

A count of zero forwards nothing and replies to nothing: the click is refused where it happens instead of opening an empty picker.

## Related

- [Desktop app](./desktop-app.md) for the window itself
- [Sessions](./sessions.md) for where a forward can land
- [Split Panes](./split-panes.md) for why a react picked in a split leaf paints on that leaf's transcript
