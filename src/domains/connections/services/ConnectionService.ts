import { Result } from "@/utils/result";
import { Connection, TestStage } from "../models";
import { FindDocumentsResult } from "../models/findDocumentsResult";

export abstract class ConnectionService {
  abstract listConnections(): Promise<Result<Connection[]>>;
  abstract getConnection(id: string): Promise<Result<Connection>>;
  abstract saveConnection(id: string, connection: Omit<Connection, "id">): Promise<Result<void>>;
  abstract saveConnections(connections: Omit<Connection, "id">[]): Promise<Result<void>>;
  abstract deleteConnection(id: string): Promise<Result<void>>;
  abstract createConnection(connection: Omit<Connection, "id">): Promise<Result<Connection>>;
  abstract updateConnection(id: string, connection: Connection): Promise<Result<void>>;
  abstract getConnectionPassword(id: string): Promise<Result<string | null>>;
  abstract testConnection(connection: Omit<Connection, 'id'>, options?: { signal?: AbortSignal }): Promise<TestStage[]>;
  abstract connect(connection: Connection, options?: { signal?: AbortSignal }): Promise<string[]>;
  abstract findDocuments(connection: Connection, databaseName: string, collectionName: string, filter: Record<string, unknown>, skip: number, limit: number, options?: { signal?: AbortSignal }): Promise<FindDocumentsResult>;
}
