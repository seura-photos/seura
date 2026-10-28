# Regional formats checked on Windows (ACC-6)

- **Nevada checked the installed 0.2.1** on Windows, with the display language in English and the regional format set to English (Germany). Claude Code wrote the list of what to look at.
  - **Dates:** day first, with English month names: `12 Jul 2026, 18:42`. That's the standard English (Germany) format (CLDR's `en-DE`). It's not the spec's example `12.07.2026`, which is German (Germany), `de-DE`. A unit test already covers that format.
  - **Numbers:** German separators throughout: `1.214` in the counts, `2,1 MB`, decimal commas in the camera details, `100 %` in the Viewer.
  - **The filter's date fields,** drawn by the web view itself, also followed the region: day first, with the calendar's week starting on Monday. That settles the possible gap raised when the filters were built (2026-10-26-filters.md). No known limitation needs recording for Windows.
- **German (Germany) wasn't offered** in Windows' list without a German language pack, Nevada found. English (Germany) is also the more likely setting for an English-speaking user in Germany, so it was the better case to check.
- **On the Mac, Nevada found it looks correct too,** with English as the language and the region set to Germany. There, the region comes from macOS's `@rg=` override.
