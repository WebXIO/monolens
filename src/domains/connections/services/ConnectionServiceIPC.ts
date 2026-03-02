import { abortableInvoke } from "@/composables/useAbortableCommand";
import { Result, tryCatch } from "@/utils/result";
import { invoke } from "@tauri-apps/api/core";
import { Connection, TestStage } from "../models";
import { ConnectionService } from "./ConnectionService";
import { FindDocumentsResult } from "../models/findDocumentsResult";

export class ConnectionServiceIPC implements ConnectionService {
   listConnections(): Promise<Result<Connection[]>> {
      return tryCatch<Connection[]>(invoke('get_connections'));
   }
   getConnection(id: string): Promise<Result<Connection>> {
      return tryCatch<Connection>(invoke('get_connection', { id }));
   }
   saveConnection(id: string, connection: Omit<Connection, "id">): Promise<Result<void>> {
      return tryCatch<void>(invoke('save_connection', { id, connection }));
   }
   saveConnections(connections: Omit<Connection, "id">[]): Promise<Result<void>> {
      return tryCatch<void>(invoke('save_connections', { connections }));
   }
   deleteConnection(id: string): Promise<Result<void>> {
      return tryCatch<void>(invoke('delete_connection', { id }));
   }
   createConnection(connection: Omit<Connection, "id">): Promise<Result<Connection>> {
      return tryCatch<Connection>(invoke('create_connection', { connection }));
   }
   updateConnection(id: string, connection: Connection): Promise<Result<void>> {
      return tryCatch<void>(invoke('update_connection', { id, connection }));
   }
   getConnectionPassword(id: string): Promise<Result<string | null>> {
      return tryCatch<string | null>(invoke('get_connection_password', { id }));
   }
   testConnection(connection: Omit<Connection, 'id'>, options?: { signal?: AbortSignal }): Promise<TestStage[]> {
      return abortableInvoke<TestStage[]>(
         'start_test_connection',
         { connection: { ...connection, id: '' } },
         { signal: options?.signal },
      );
   }
   connect(connection: Connection, options?: { signal?: AbortSignal }): Promise<string[]> {
      return abortableInvoke<string[]>(
         'start_connect',
         { connection },
         { signal: options?.signal },
      );
   }
   findDocuments(
      connection: Connection,
      databaseName: string,
      collectionName: string,
      filter: Record<string, unknown>,
      skip: number,
      limit: number,
      options?: { signal?: AbortSignal }
   ): Promise<FindDocumentsResult> {
      return abortableInvoke<FindDocumentsResult>(
         'start_find_documents',
         { connection, databaseName, collectionName, filter, skip, limit },
         { signal: options?.signal },
      );
   }
}