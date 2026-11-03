import { describe, expect, it } from "vitest";
import { eventToKeyString } from "../utils";

function key(key: string, mods: Partial<Pick<KeyboardEvent, "ctrlKey" | "metaKey" | "shiftKey" | "altKey">> = {}) {
  return { key, ctrlKey: false, metaKey: false, shiftKey: false, altKey: false, ...mods } as KeyboardEvent;
}

describe("eventToKeyString", () => {
  it("maps Ctrl to mod outside macOS", () => {
    expect(eventToKeyString(key("w", { ctrlKey: true }), false)).toBe("mod+w");
  });

  it("maps Cmd to mod on macOS", () => {
    expect(eventToKeyString(key("w", { metaKey: true }), true)).toBe("mod+w");
  });

  it("keeps Ctrl apart from mod on macOS", () => {
    expect(eventToKeyString(key("w", { ctrlKey: true }), true)).toBe("ctrl+w");
  });

  it("keeps the meta key apart from mod outside macOS", () => {
    expect(eventToKeyString(key("w", { metaKey: true }), false)).toBe("meta+w");
  });

  it("orders modifiers and lowercases the key", () => {
    expect(eventToKeyString(key("D", { ctrlKey: true, shiftKey: true }), false)).toBe("mod+shift+d");
    expect(eventToKeyString(key("Enter", { ctrlKey: true, altKey: true, shiftKey: true }), false)).toBe("mod+shift+alt+enter");
  });

  it("returns only the modifiers while a modifier key itself is pressed", () => {
    expect(eventToKeyString(key("Control", { ctrlKey: true }), false)).toBe("mod");
    expect(eventToKeyString(key("Shift", { shiftKey: true }), false)).toBe("shift");
    expect(eventToKeyString(key("Meta", { metaKey: true }), true)).toBe("mod");
  });

  it("returns the key alone without modifiers", () => {
    expect(eventToKeyString(key("a"), false)).toBe("a");
  });
});
