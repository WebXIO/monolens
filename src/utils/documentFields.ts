/** The top-level field names of all documents, `_id` first, then alphabetically. Table columns and autocomplete use it. */
export function fieldNames(documents: Record<string, unknown>[]): string[] {
  const names = new Set<string>();
  for (const document of documents) {
    for (const key of Object.keys(document)) names.add(key);
  }
  return [...names].sort((a, b) => (a === '_id' ? -1 : b === '_id' ? 1 : a.localeCompare(b)));
}
