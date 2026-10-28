import { type KeyboardEvent, useEffect, useRef, useState } from "react";
import mark from "../../design/brand/seura-mark.svg";
import { commands, type Link } from "./bindings";
import { t } from "./messages";
import { useStore } from "./store";

const LINKS: [Link, string][] = [
  ["website", "about-website"],
  ["project", "about-project"],
  ["license", "about-license"],
];

/**
 * About Seura Photos on Windows (SET-5), from the ⋯ button: the version,
 * selectable and with Copy, so a problem report can say which build it's
 * about; while below 1.0, that it's an early build; and links to the
 * website, the project, and the license, opened in the browser. On macOS
 * the system's About panel shows the same. OK has the focus, and Esc
 * closes it; the focus then goes back where it was.
 */
export function About() {
  const open = useStore((state) => state.about);
  const version = useStore((state) => state.version);
  const setAbout = useStore((state) => state.setAbout);
  // What Copy did, and why a link didn't open, if one didn't.
  const [copied, setCopied] = useState<string | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const dialog = useRef<HTMLDivElement>(null);
  const ok = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!open) return;
    const before = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    ok.current?.focus();
    return () => {
      setCopied(null);
      setProblem(null);
      if (before?.isConnected) before.focus();
    };
  }, [open]);

  if (!open || version === null) return null;

  async function copy() {
    try {
      await navigator.clipboard.writeText(version ?? "");
      setCopied(t("about-copied"));
    } catch (error) {
      setCopied(t("about-copy-failed", { reason: String(error) }));
    }
  }

  async function openLink(link: Link) {
    const result = await commands.openLink(link);
    setProblem(result.status === "error" ? t("about-link-failed", { reason: result.error }) : null);
  }

  function onKeyDown(event: KeyboardEvent) {
    // The views behind don't hear its keys: a number would rate the
    // selected photos, Esc would close the Viewer.
    event.stopPropagation();
    if (event.key === "Escape") {
      event.preventDefault();
      setAbout(false);
    } else if (event.key === "Tab") {
      // The focus stays in the dialog.
      const all = [...(dialog.current?.querySelectorAll<HTMLElement>("button") ?? [])];
      const at = all.indexOf(document.activeElement as HTMLElement);
      const next = event.shiftKey ? (at <= 0 ? all.length - 1 : at - 1) : (at + 1) % all.length;
      event.preventDefault();
      all[next]?.focus();
    }
  }

  return (
    <div className="dialog-backdrop">
      <div
        ref={dialog}
        className="dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="about-title"
        onKeyDown={onKeyDown}
      >
        <div className="about-heading">
          <img className="about-mark" src={mark} alt="" />
          <h2 id="about-title" className="dialog-title">
            {t("app-name")}
          </h2>
        </div>
        <p className="about-version">
          <span className="about-version-text">{t("app-version", { version })}</span>
          <button type="button" className="button" onClick={() => void copy()}>
            {t("about-copy")}
          </button>
          <output>{copied}</output>
        </p>
        {version.startsWith("0.") && <p>{t("about-early-build")}</p>}
        <ul className="about-links">
          {LINKS.map(([link, label]) => (
            <li key={link}>
              <button
                type="button"
                role="link"
                className="link-button"
                onClick={() => void openLink(link)}
              >
                {t(label)}
              </button>
            </li>
          ))}
        </ul>
        {problem !== null && <p role="alert">{problem}</p>}
        <div className="dialog-actions">
          <button ref={ok} type="button" className="button primary" onClick={() => setAbout(false)}>
            {t("about-ok")}
          </button>
        </div>
      </div>
    </div>
  );
}
