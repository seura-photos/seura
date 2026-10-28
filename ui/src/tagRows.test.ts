import { describe, expect, it } from "vitest";
import type { TagInfo } from "./bindings";
import { tagAndBeneath, tagPathTo } from "./tagRows";

const node = (tag: string, parent: string | null): TagInfo => ({
  tag,
  parent,
  name: tag.split("/").pop() ?? tag,
  photos: 1,
});

const tags = [
  node("family/kids/school", "family/kids"),
  node("family", null),
  node("family/kids", "family"),
  node("family/pets", "family"),
  node("travel", null),
];

describe("Collapse all and Expand all beneath (TAG-5)", () => {
  it("keeps open only the way to the selected tag", () => {
    expect([...tagPathTo(tags, "family/kids/school")].sort()).toEqual(["family", "family/kids"]);
    expect([...tagPathTo(tags, "travel")]).toEqual([]);
    expect([...tagPathTo(tags, null)]).toEqual([]);
  });

  it("opens a tag and every tag beneath it that has tags beneath it", () => {
    expect(tagAndBeneath(tags, "family").sort()).toEqual(["family", "family/kids"]);
    expect(tagAndBeneath(tags, "travel")).toEqual([]);
  });
});
