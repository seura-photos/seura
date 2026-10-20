# Who does what: design sessions and Claude Code

The project now runs in two kinds of Claude sessions, and Nevada settled which one owns what.

- **Design sessions** (Nevada with Claude in the Claude app) own the product and design documents: the brief, the functional spec, the UX documents, the visual design, the technical design, and the edit format. Changes arrive as pull requests that Nevada merges.
- **Claude Code** (on Nevada's Windows PC, with a second session on the MacBook Pro for checks) implements them. When the implementation shows a gap or a contradiction, it raises it rather than changing a document's meaning. Small factual corrections the implementation settled, such as a library version or a measured number, can go in its own pull requests. Example: in M1 it found three gaps in the edit format (newer minor versions, redo after a restart, deleting empty edit files) and raised them; the format was fixed in a design session.
- **The website** is split the same way. Claude Code writes the devlog posts alongside the app work, and the design sessions handle everything else: pages, layout, SEO, social cards. Nevada asked for this split after the site launched.
- **The journal is shared.** Both kinds of session add entries here, and ideas from outside the project count too (a friend's suggestion led to the highlights and shadows sliders). Claude Code reads recent entries before drafting a post.
- **Everything goes through pull requests**, on protected main branches. Nevada and Claude can bypass the protection for a direct push when needed.
- **Why it matters:** separate sessions don't share memory, so the repositories are the shared memory. Each repository's CLAUDE.md says who changes what, and every Claude session reads it on start.
