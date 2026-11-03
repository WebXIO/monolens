import { Result } from "@/utils/result";
import { Connection, TestStage } from "../models";
import { FindDocumentsResult, FindQuery } from "../models/findDocumentsResult";

export abstract class ConnectionService {
  abstract listConnections(): Promise<Result<Connection[]>>;
  abstract deleteConnection(id: string): Promise<Result<void>>;
  abstract createConnection(connection: Omit<Connection, "id">): Promise<Result<Connection>>;
  abstract updateConnection(id: string, connection: Connection): Promise<Result<void>>;
  abstract getConnectionPassword(id: string): Promise<Result<string | null>>;
  abstract testConnection(connection: Omit<Connection, 'id'>, testId: string, options?: { signal?: AbortSignal }): Promise<TestStage[]>;
  abstract connect(connectionId: string, options?: { signal?: AbortSignal }): Promise<string[]>;
  abstract disconnect(connectionId: string): Promise<Result<void>>;
  abstract listCollections(connectionId: string, databaseName: string): Promise<Result<string[]>>;
  abstract findDocuments(connectionId: string, query: FindQuery, options?: { signal?: AbortSignal }): Promise<FindDocumentsResult>;
}
