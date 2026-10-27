/**
 * Dates, times, numbers, and sizes in the operating system's region
 * (ACC-6), even though the interface is in English. The core reports the
 * region (`commands.region()`); the web view's own locale follows the
 * display language instead, so it can't be used. The core passes raw
 * values, and only these functions turn them into text.
 */

let region: string | undefined;

/** Sets the region for every format. Invalid tags fall back to the web view's. */
export function setRegion(tag: string) {
  try {
    region = Intl.getCanonicalLocales(tag)[0];
  } catch {
    region = undefined;
  }
}

export function formatNumber(value: number, options?: Intl.NumberFormatOptions): string {
  return new Intl.NumberFormat(region, options).format(value);
}

/** A file size in bytes, as "2.1 MB" or "2,1 MB" (decimal units, like Explorer and Finder). */
export function formatBytes(bytes: number): string {
  const units = ["byte", "kilobyte", "megabyte", "gigabyte", "terabyte"] as const;
  let value = bytes;
  let unit = 0;
  while (value >= 1000 && unit < units.length - 1) {
    value /= 1000;
    unit += 1;
  }
  return formatNumber(value, {
    style: "unit",
    unit: units[unit],
    unitDisplay: unit === 0 ? "long" : "short",
    maximumFractionDigits: unit === 0 || value >= 100 ? 0 : 1,
  });
}

/**
 * A date and time, such as "Jul 12, 2026, 6:42 PM" or "12.07.2026, 18:42".
 * Photos record the time on their own clock with no time zone, so a date
 * taken is shown as it was recorded, not converted. A file's own times are
 * real moments, shown in the computer's time zone (`local`).
 */
export function formatDateTime(
  secondsSince1970: number,
  { local = false }: { local?: boolean } = {},
): string {
  return new Intl.DateTimeFormat(region, {
    dateStyle: "medium",
    timeStyle: "short",
    ...(local ? {} : { timeZone: "UTC" }),
  }).format(new Date(secondsSince1970 * 1000));
}

/** A date taken, without the time, such as "Jul 12, 2026", as recorded. */
export function formatDate(secondsSince1970: number): string {
  return new Intl.DateTimeFormat(region, { dateStyle: "medium", timeZone: "UTC" }).format(
    new Date(secondsSince1970 * 1000),
  );
}
