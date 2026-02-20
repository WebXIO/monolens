import { Result, tryCatch } from "@/utils/result";
import { invoke } from "@tauri-apps/api/core";
import { Connection } from "../models";
import { ConnectionService } from "./ConnectionService";

export class ConnectionServiceIPC implements ConnectionService {
   listConnections(): Promise<Result<Connection[]>> {
      return tryCatch<Connection[]>(invoke('get_connections'));
   }
   getConnection(id: string): Promise<Result<Connection>> {
      return tryCatch<Connection>(invoke('get_connection', {id}));
   }
   saveConnection(id: string, connection: Omit<Connection, "id">): Promise<Result<void>> {
      return tryCatch<void>(invoke('save_connection', {id, connection}));
   }
   saveConnections(connections: Omit<Connection, "id">[]): Promise<Result<void>> {
      return tryCatch<void>(invoke('save_connections', {connections}));
   }
   deleteConnection(id: string): Promise<Result<void>> {
      return tryCatch<void>(invoke('delete_connection', {id}));
   }
   createConnection(connection: Omit<Connection, "id">): Promise<Result<Connection>> {
      return tryCatch<Connection>(invoke('create_connection', {connection}));
   }
   updateConnection(id: string, connection: Connection): Promise<Result<void>> {
      return tryCatch<void>(invoke('update_connection', {id, connection}));
   }
   getConnectionPassword(id: string): Promise<Result<string | null>> {
      return tryCatch<string | null>(invoke('get_connection_password', {id}));
   }
}