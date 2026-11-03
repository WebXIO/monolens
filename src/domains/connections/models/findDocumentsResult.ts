export interface FindQuery {
  databaseName: string;
  collectionName: string;
  /** extended JSON objects from the query parser; {} or null means none */
  filter: Record<string, unknown>;
  sort?: Record<string, unknown> | null;
  projection?: Record<string, unknown> | null;
  skip: number;
  /** 1 to 10000 */
  limit: number;
}

export interface FindDocumentsResult {
  documents: Record<string, unknown>[];
  totalCount: number;
  /** an empty filter is counted from the collection's metadata, which is fast but may be off */
  totalCountEstimated: boolean;
}
