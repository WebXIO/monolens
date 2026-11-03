import { parseMongoDbQuery } from '@/utils/queryParser';

export type QueryTexts = { filter: string; sort: string; projection: string };
export type ParsedQuery = {
  filter: Record<string, unknown>;
  sort: Record<string, unknown> | null;
  projection: Record<string, unknown> | null;
};

const PART_NAMES: Record<keyof QueryTexts, string> = { filter: 'Filter', sort: 'Sort', projection: 'Projection' };

function parsePart(name: keyof QueryTexts, text: string): Record<string, unknown> | null {
  if (!text.trim()) return null;
  let value: unknown;
  try {
    value = parseMongoDbQuery(text);
  } catch (err) {
    throw new Error(`${PART_NAMES[name]}: ${err instanceof Error ? err.message : String(err)}`);
  }
  if (value === null || typeof value !== 'object' || Array.isArray(value)) {
    throw new Error(`${PART_NAMES[name]}: must be an object like { field: value }`);
  }
  const object = value as Record<string, unknown>;
  return Object.keys(object).length > 0 ? object : null;
}

/** The three inputs of the query tab; empty or `{ }` means none. An error names the input. */
export function parseQueryTexts(texts: QueryTexts): ParsedQuery {
  return {
    filter: parsePart('filter', texts.filter) ?? {},
    sort: parsePart('sort', texts.sort),
    projection: parsePart('projection', texts.projection),
  };
}

/** Field names of all documents for the autocomplete, `_id` first. */
export function fieldNames(documents: Record<string, unknown>[]): string[] {
  const names = new Set<string>();
  for (const document of documents) {
    for (const key of Object.keys(document)) names.add(key);
  }
  return [...names].sort((a, b) => (a === '_id' ? -1 : b === '_id' ? 1 : a.localeCompare(b)));
}

export type PageInfo = {
  /** "51–100 of 137", "~" before an estimated total */
  label: string;
  hasPrevious: boolean;
  hasNext: boolean;
};

export function pageInfo(skip: number, shown: number, total: number, estimated: boolean, limit: number): PageInfo {
  const totalText = `${estimated ? '~' : ''}${total.toLocaleString('en-US')}`;
  const label = shown === 0
    ? `0 of ${totalText}`
    : `${(skip + 1).toLocaleString('en-US')}–${(skip + shown).toLocaleString('en-US')} of ${totalText}`;
  return {
    label,
    hasPrevious: skip > 0,
    // a full page may be followed by more even when an estimate says otherwise
    hasNext: skip + shown < total || (estimated && shown === limit),
  };
}
