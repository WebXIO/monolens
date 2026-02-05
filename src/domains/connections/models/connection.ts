import { AuthenticationOptions } from "./authentication-options";
import { ConnectionType } from "./connection-type";

export type Connection = {
  id: string;
  name: string;
  uri: string;
  port: number;
  connector: string;
  connectionType: ConnectionType;
  authentication: AuthenticationOptions;
};
