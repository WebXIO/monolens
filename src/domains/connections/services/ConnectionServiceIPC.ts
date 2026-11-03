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
   // the progress events of this test carry testId
   testConnection(connection: Omit<Connection, 'id'>, testId: string, options?: { signal?: AbortSignal }): Promise<TestStage[]> {
      return abortableInvoke<TestStage[]>(
         'start_test_connection',
         { connection: { ...connection, id: '' }, testId },
         { signal: options?.signal },
      );
   }
   // the rust core loads the saved connection and its password by id
   connect(connectionId: string, options?: { signal?: AbortSignal }): Promise<string[]> {
      return abortableInvoke<string[]>(
         'start_connect',
         { connectionId },
         { signal: options?.signal },
      );
   }
   disconnect(connectionId: string): Promise<Result<void>> {
      return tryCatch<void>(invoke('disconnect', { connectionId }));
   }
   listCollections(connectionId: string, databaseName: string): Promise<Result<string[]>> {
      return tryCatch<string[]>(invoke('list_collections', { connectionId, databaseName }));
   }
   findDocuments(
      connectionId: string,
      databaseName: string,
      collectionName: string,
      filter: Record<string, unknown>,
      skip: number,
      limit: number,
      options?: { signal?: AbortSignal }
   ): Promise<FindDocumentsResult> {
      return abortableInvoke<FindDocumentsResult>(
         'start_find_documents',
         { connectionId, databaseName, collectionName, filter, skip, limit },
         { signal: options?.signal },
      );
   }
}
