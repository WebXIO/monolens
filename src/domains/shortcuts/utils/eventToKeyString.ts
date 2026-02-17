export function eventToKeyString(ev: KeyboardEvent): string {
  const parts: string[] = [];

  if (ev.ctrlKey) parts.push("ctrl");
  if (ev.shiftKey) parts.push("shift");
  if (ev.altKey) parts.push("alt");
  if (ev.metaKey) parts.push("meta");

  const key = ev.key.toLowerCase();

  if (!['control', 'shift', 'alt', 'meta'].includes(key)) {
   parts.push(key);
  }

  return parts.join('+');
}
