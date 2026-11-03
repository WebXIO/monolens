import { AuthenticationOptions } from "./authentication-options";

export enum ConnectorKind {
  MongoDb = "MongoDb"
}

/** Host and port with option fields, or a mongodb:// / mongodb+srv:// string stored without credentials */
export type ConnectionTarget =
  | { kind: "Host"; host: string; port: number }
  | { kind: "Uri"; uri: string };

/** Used for a Host target only; a connection string carries its own options */
export type ConnectionOptions = {
  tls: boolean;
  tlsCaFile: string | null;
  tlsAllowInvalidCertificates: boolean;
  replicaSet: string | null;
  /** null means the driver's default (30 s) */
  serverSelectionTimeoutSecs: number | null;
};

export type Connection = {
  id: string;
  name: string;
  connector: ConnectorKind;
  target: ConnectionTarget;
  authentication: AuthenticationOptions;
  options: ConnectionOptions;
};

export function defaultConnectionOptions(): ConnectionOptions {
  return {
    tls: false,
    tlsCaFile: null,
    tlsAllowInvalidCertificates: false,
    replicaSet: null,
    serverSelectionTimeoutSecs: null,
  };
}

/** What the UI shows as the address of a connection */
export function describeTarget(target: ConnectionTarget): string {
  return target.kind === "Host" ? `${target.host}:${target.port}` : target.uri;
}
