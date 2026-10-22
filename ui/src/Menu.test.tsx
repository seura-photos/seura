import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Menu } from "./Menu";
import { suppressWebViewMenu } from "./webViewMenu";

/** A button that opens a menu of three items, the second disabled. */
function Opener() {
  const [open, setOpen] = useState(false);
  return (
    <>
      <button type="button" onClick={() => setOpen(true)}>
        Photo
      </button>
      {open && (
        <Menu x={0} y={0} onClose={() => setOpen(false)}>
          <button type="button" role="menuitem" className="menu-item">
            First
          </button>
          <button type="button" role="menuitem" className="menu-item" disabled>
            Unavailable
          </button>
          <button type="button" role="menuitem" className="menu-item">
            Last
          </button>
        </Menu>
      )}
    </>
  );
}

describe("Menu (ACC-1)", () => {
  afterEach(cleanup);

  it("moves between items with the arrow keys, skipping disabled ones", () => {
    render(<Opener />);
    const opener = screen.getByRole("button", { name: "Photo" });
    opener.focus();
    fireEvent.click(opener);
    const first = screen.getByRole("menuitem", { name: "First" });
    const last = screen.getByRole("menuitem", { name: "Last" });
    expect(document.activeElement).toBe(first);
    fireEvent.keyDown(first, { key: "ArrowDown" });
    expect(document.activeElement).toBe(last);
    fireEvent.keyDown(last, { key: "ArrowDown" });
    expect(document.activeElement).toBe(first);
    fireEvent.keyDown(first, { key: "ArrowUp" });
    expect(document.activeElement).toBe(last);
    fireEvent.keyDown(last, { key: "Home" });
    expect(document.activeElement).toBe(first);
  });

  it("gives the focus back when Esc closes it, so the menu key works again", () => {
    render(<Opener />);
    const opener = screen.getByRole("button", { name: "Photo" });
    opener.focus();
    fireEvent.click(opener);
    fireEvent.keyDown(screen.getByRole("menu"), { key: "Escape" });
    expect(screen.queryByRole("menu")).toBeNull();
    expect(document.activeElement).toBe(opener);
  });

  it("doesn't let the keys it uses reach the page", () => {
    const outside = vi.fn();
    window.addEventListener("keydown", outside);
    render(<Opener />);
    fireEvent.click(screen.getByRole("button", { name: "Photo" }));
    fireEvent.keyDown(screen.getByRole("menuitem", { name: "First" }), { key: "Escape" });
    window.removeEventListener("keydown", outside);
    expect(outside).not.toHaveBeenCalled();
  });
});

describe("the web view's own menu", () => {
  it("is turned off except in text fields", () => {
    const page = document.implementation.createHTMLDocument();
    page.body.innerHTML = '<div id="photo"></div><input id="name" />';
    suppressWebViewMenu(page);
    const open = (id: string) => {
      const event = new MouseEvent("contextmenu", { bubbles: true, cancelable: true });
      page.getElementById(id)?.dispatchEvent(event);
      return !event.defaultPrevented;
    };
    expect(open("photo")).toBe(false);
    expect(open("name")).toBe(true);
  });
});
