import { AuthenticationOptions } from "./authentication-options";
import { ConnectionType } from "./connection-type";

export enum ConnectorKind {
  MongoDb = "MongoDb"
}

export type Connection = {
  id: string;
  name: string;
  uri: string;
  port: number;
  connector: ConnectorKind;
  connectionType: ConnectionType;
  authentication: AuthenticationOptions;
};
