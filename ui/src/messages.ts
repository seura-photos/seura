// All UI text lives in the message catalogs in ./messages, never in components,
// so translations can be added later (ACC-5). English only in v1.
import en from "./messages/en.json";

export type MessageKey = keyof typeof en;

/** The text for `key`, with `{name}` placeholders filled from `values`. */
export function t(key: MessageKey, values: Record<string, string> = {}): string {
  return en[key].replace(/\{(\w+)\}/g, (placeholder, name: string) => values[name] ?? placeholder);
}
