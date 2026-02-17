import { Result } from "@/utils/result";
import { Connection } from "../models";
import { ConnectionRepository } from "./ConnectionRepository";
import { ConnectionService } from "../services";

export class ConnectionRepositoryIPC implements ConnectionRepository {
  connectionService: ConnectionService;
  cachedConnections?: Connection[];

  constructor(params: { connectionService: ConnectionService }) {
    this.connectionService = params.connectionService;
  }

  listConnections(): Promise<Result<Connection[]>> {
    return this.connectionService.listConnections();
  }
  getConnection(id: string): Promise<Result<Connection>> {
    return this.connectionService.getConnection(id);
  }
  saveConnection(
    id: string,
    connection: Omit<Connection, "id">,
  ): Promise<Result<void>> {
    return this.connectionService.saveConnection(id, connection);
  }
  saveConnections(
    connections: Omit<Connection, "id">[],
  ): Promise<Result<void>> {
    return this.connectionService.saveConnections(connections);
  }
  deleteConnection(id: string): Promise<Result<void>> {
    return this.connectionService.deleteConnection(id);
  }
  createConnection(
    connection: Omit<Connection, "id">,
  ): Promise<Result<Connection>> {
    return this.connectionService.createConnection(connection);
  }
  updateConnection(
    id: string,
    connection: Connection,
  ): Promise<Result<void>> {
    return this.connectionService.saveConnection(id, connection);
  }
}
