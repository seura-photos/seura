# Revert pinned, built (Claude Code)

- **Built as the design session settled it (#176),** on its own branch from `main`, not after slice 6, so it doesn't wait for effects.
- **Cancel stays in the footer, exactly where the Revert button was.** The card holds only the question and Revert, and sits just above the footer, over the scrolling sections. Cancel comes first in the page, so Tab goes from Cancel to Revert as the visual design asks, while the eye reads the card above it first.
- **The scrolling shadows** come from a small `ScrollArea` component. It checks the scroll position as the user scrolls and as the panel or its contents change size, for example when an effect's sliders appear. Crop mode's panel scrolls the same way.
- **Not yet seen in the real app.** The tests check the structure: what's pinned, where Cancel is, the card, Tab order, a click outside. The layout itself needs Nevada's eye on a short window.
