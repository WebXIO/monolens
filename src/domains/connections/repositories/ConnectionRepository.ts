import { Result } from "@/utils/result";
import { Connection } from "../models";

export interface ConnectionRepository {
  listConnections(): Promise<Result<Connection[]>>;
  getConnection(id: string): Promise<Result<Connection>>;
  saveConnection(
    id: string,
    connection: Omit<Connection, "id">,
  ): Promise<Result<void>>;
  saveConnections(connections: Omit<Connection, "id">[]): Promise<Result<void>>;
  deleteConnection(id: string): Promise<Result<void>>;
  createConnection(
    connection: Omit<Connection, "id">,
  ): Promise<Result<Connection>>;
  updateConnection(
    id: string,
    connection: Connection,
  ): Promise<Result<void>>;
}
