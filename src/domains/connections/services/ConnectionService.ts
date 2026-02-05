import { Result } from "@/utils/result";
import { Connection } from "../models";

export abstract class ConnectionService {
  abstract listConnections(): Promise<Result<Connection[]>>;
  abstract getConnection(id: string): Promise<Result<Connection>>;
  abstract saveConnection(id: string, connection: Omit<Connection, "id">): Promise<Result<void>>;
  abstract saveConnections(connections: Omit<Connection, "id">[]): Promise<Result<void>>;
  abstract deleteConnection(id: string): Promise<Result<void>>;
  abstract createConnection(connection: Omit<Connection, "id">): Promise<Result<Connection>>;
}
