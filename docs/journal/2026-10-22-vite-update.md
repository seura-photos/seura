# Vite 8.3.4, once it was two weeks old

- **The advisory:** Vite 8.3.2's development server had a high-severity advisory (its "launch editor" endpoint), found during M2 slice 1. The fix, 8.3.4, came out on 2026-10-08, and the project's rule is to use versions at least two weeks old, so the update waited until 2026-10-22.
- **Updated** to exactly 8.3.4 (8.3.5, from 2026-10-21, is too new): `npm audit` now finds nothing, and the UI's licenses, build, and tests pass. The advisory only ever affected the development server, never the built app.
