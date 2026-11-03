export function isMacPlatform(): boolean {
  return typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.userAgent);
}

// "mod" is the platform's command key: Cmd on macOS, Ctrl everywhere else
export function eventToKeyString(ev: KeyboardEvent, isMac = isMacPlatform()): string {
  const parts: string[] = [];

  if (isMac ? ev.metaKey : ev.ctrlKey) parts.push("mod");
  if (isMac && ev.ctrlKey) parts.push("ctrl");
  if (!isMac && ev.metaKey) parts.push("meta");
  if (ev.shiftKey) parts.push("shift");
  if (ev.altKey) parts.push("alt");

  const key = ev.key.toLowerCase();

  if (!['control', 'shift', 'alt', 'meta'].includes(key)) {
   parts.push(key);
  }

  return parts.join('+');
}
