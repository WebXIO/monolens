import { z } from 'zod';
import {
  AuthenticationKind,
  ConnectorKind,
  type Connection,
  type ConnectionOptions,
} from '@/domains/connections';

export type ConnectionMode = 'host' | 'uri';

const CONNECTION_STRING = /^mongodb(\+srv)?:\/\/\S+$/;
const MAX_TIMEOUT_SECS = 3600;

/** Every field of the dialog in both modes; the mode decides which ones count. */
export const connectionFormSchema = z
  .object({
    name: z.string().trim().min(1, 'Name is required'),
    mode: z.enum(['host', 'uri']),
    host: z.string(),
    port: z.number(),
    uri: z.string(),
    useAuth: z.boolean(),
    username: z.string(),
    password: z.string(),
    authDatabase: z.string(),
    tls: z.boolean(),
    tlsCaFile: z.string(),
    tlsAllowInvalidCertificates: z.boolean(),
    replicaSet: z.string(),
    /** seconds as typed; empty means the driver's default */
    timeoutSecs: z.string(),
  })
  .superRefine((values, ctx) => {
    const issue = (path: string, message: string) => ctx.addIssue({ code: 'custom', path: [path], message });

    if (values.mode === 'host') {
      const host = values.host.trim();
      if (!host) issue('host', 'Host is required');
      else if (host.includes('://')) issue('host', 'Use "Connection string" for mongodb:// and mongodb+srv:// addresses');

      if (!Number.isInteger(values.port) || values.port < 1 || values.port > 65535) {
        issue('port', 'Port must be between 1 and 65535');
      }

      const timeout = values.timeoutSecs.trim();
      if (timeout && (!/^\d+$/.test(timeout) || Number(timeout) < 1 || Number(timeout) > MAX_TIMEOUT_SECS)) {
        issue('timeoutSecs', `Seconds from 1 to ${MAX_TIMEOUT_SECS}`);
      }
    } else if (!CONNECTION_STRING.test(values.uri.trim())) {
      issue('uri', 'A connection string starts with mongodb:// or mongodb+srv://');
    }

    if (values.useAuth) {
      if (!values.username.trim()) issue('username', 'Username is required');
      if (!values.password) issue('password', 'Password is required');
    }
  });

export type ConnectionFormValues = z.infer<typeof connectionFormSchema>;

export function defaultFormValues(): ConnectionFormValues {
  return {
    name: '',
    mode: 'host',
    host: 'localhost',
    port: 27017,
    uri: '',
    useAuth: false,
    username: '',
    password: '',
    authDatabase: 'admin',
    tls: false,
    tlsCaFile: '',
    tlsAllowInvalidCertificates: false,
    replicaSet: '',
    timeoutSecs: '',
  };
}

/** The dialog's values for a saved connection; the password comes from the keyring separately. */
export function formValuesFromConnection(connection: Connection, password: string): ConnectionFormValues {
  const { target, authentication, options } = connection;
  const defaults = defaultFormValues();

  return {
    ...defaults,
    name: connection.name,
    mode: target.kind === 'Host' ? 'host' : 'uri',
    host: target.kind === 'Host' ? target.host : defaults.host,
    port: target.kind === 'Host' ? target.port : defaults.port,
    uri: target.kind === 'Uri' ? target.uri : '',
    useAuth: authentication.kind !== AuthenticationKind.NONE,
    username: authentication.username ?? '',
    password,
    authDatabase: authentication.database ?? defaults.authDatabase,
    tls: options.tls,
    tlsCaFile: options.tlsCaFile ?? '',
    tlsAllowInvalidCertificates: options.tlsAllowInvalidCertificates,
    replicaSet: options.replicaSet ?? '',
    timeoutSecs: options.serverSelectionTimeoutSecs === null ? '' : String(options.serverSelectionTimeoutSecs),
  };
}

function orNull(value: string): string | null {
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
}

/**
 * What the dialog saves or tests. A connection string carries its own options, so the option
 * fields only count in host mode. Credentials typed into the string are moved out by the Rust
 * core on save.
 */
export function connectionFromFormValues(values: ConnectionFormValues): Omit<Connection, 'id'> {
  const options: ConnectionOptions = values.mode === 'host'
    ? {
      tls: values.tls,
      tlsCaFile: values.tls ? orNull(values.tlsCaFile) : null,
      tlsAllowInvalidCertificates: values.tls && values.tlsAllowInvalidCertificates,
      replicaSet: orNull(values.replicaSet),
      serverSelectionTimeoutSecs: values.timeoutSecs.trim() ? Number(values.timeoutSecs.trim()) : null,
    }
    : {
      tls: false,
      tlsCaFile: null,
      tlsAllowInvalidCertificates: false,
      replicaSet: null,
      serverSelectionTimeoutSecs: null,
    };

  return {
    name: values.name.trim(),
    connector: ConnectorKind.MongoDb,
    target: values.mode === 'host'
      ? { kind: 'Host', host: values.host.trim(), port: values.port }
      : { kind: 'Uri', uri: values.uri.trim() },
    authentication: {
      kind: values.useAuth ? AuthenticationKind.BASIC : AuthenticationKind.NONE,
      username: values.useAuth ? orNull(values.username) : null,
      password: values.useAuth && values.password ? values.password : null,
      database: values.useAuth ? orNull(values.authDatabase) : null,
    },
    options,
  };
}
