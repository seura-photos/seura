# Date ranges and the filter row (M4 slice 0b; #125, #118)

- **Date ranges** use `Intl.DateTimeFormat`'s `formatRange` in the region, as the design session answered. Three photos taken in the same minute now read "Mar 26, 2007, 12:51 PM", not the same time twice; a few hours apart, "Mar 26, 2007, 12:51 – 3:05 PM"; in a German region, "26.03.2007, 12:51 – 05.05.2007, 12:51". The catalog's "{ first } to { last }" is gone: the region's own range style replaces it, and it follows ACC-6.
- **The filter row:** the Tags control is a native list, which sizes itself to its longest choice. Its closed text is always "Add a tag…", so a long tag only widened it, without showing. It's now at most 200 px wide, and the open list still shows each tag in full. The row still wraps only when it must.
