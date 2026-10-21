// All UI text lives in the message catalog in ./messages, never in components,
// so translations can be added later (ACC-5). Project Fluent, as in the core,
// so plurals and word order are each language's own. English only in v1.
import { FluentBundle, FluentNumber, FluentResource, type FluentVariable } from "@fluent/bundle";
import { formatNumber } from "./format";
import english from "./messages/en.ftl?raw";
import { pseudoTranslate } from "./pseudo";

/**
 * A number shown in the operating system's region (ACC-6) while the
 * English catalog chooses the words: "1,214 photos" in the US, "1.214
 * photos" in Germany. Plural rules still come from the catalog's language.
 */
class RegionNumber extends FluentNumber {
  override toString(): string {
    return formatNumber(this.value);
  }
}

function makeBundle(pseudo: boolean): FluentBundle {
  const bundle = new FluentBundle("en", {
    // Bidirectional isolation marks are for right-to-left languages, which
    // come after v1; without them, text stays plain when copied or tested.
    useIsolating: false,
    ...(pseudo ? { transform: pseudoTranslate } : {}),
  });
  const errors = bundle.addResource(new FluentResource(english));
  if (errors.length > 0) throw new Error(`messages/en.ftl: ${errors.join("; ")}`);
  return bundle;
}

let bundle = makeBundle(false);

/** Switches pseudo-translation on or off (`SEURA_LOCALE=pseudo`, ACC-7). */
export function setPseudo(pseudo: boolean) {
  bundle = makeBundle(pseudo);
}

/** Whether the catalog has a message: for tests. */
export function hasMessage(id: string): boolean {
  return bundle.hasMessage(id);
}

/**
 * The text for message `id`, with its variables. Numbers are passed as
 * numbers, so plurals work and they're formatted in the region.
 */
export function t(id: string, values: Record<string, string | number> = {}): string {
  const message = bundle.getMessage(id);
  if (!message?.value) return id;
  const args: Record<string, FluentVariable> = {};
  for (const [name, value] of Object.entries(values)) {
    args[name] = typeof value === "number" ? new RegionNumber(value) : value;
  }
  return bundle.formatPattern(message.value, args);
}
