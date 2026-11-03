import { describe, expect, it } from "vitest";
import { AuthenticationKind, ConnectorKind, defaultConnectionOptions, type Connection } from "@/domains/connections";
import {
  connectionFormSchema,
  connectionFromFormValues,
  defaultFormValues,
  formValuesFromConnection,
  type ConnectionFormValues,
} from "../connectionForm";

function values(overrides: Partial<ConnectionFormValues> = {}): ConnectionFormValues {
  return { ...defaultFormValues(), name: "Local", ...overrides };
}

function issues(overrides: Partial<ConnectionFormValues>) {
  const result = connectionFormSchema.safeParse(values(overrides));
  return result.success ? {} : Object.fromEntries(result.error.issues.map((i) => [i.path.join("."), i.message]));
}

describe("connectionFormSchema", () => {
  it("accepts the defaults with a name", () => {
    expect(issues({})).toEqual({});
  });

  it("requires a name", () => {
    expect(issues({ name: "  " })).toHaveProperty("name");
  });

  it("checks host and port in host mode", () => {
    expect(issues({ host: " " })).toEqual({ host: "Host is required" });
    expect(issues({ host: "mongodb://h" })).toHaveProperty("host");
    expect(issues({ port: 0 })).toHaveProperty("port");
    expect(issues({ port: 65536 })).toHaveProperty("port");
    expect(issues({ port: 1.5 })).toHaveProperty("port");
  });

  it("checks the timeout only when one is given", () => {
    expect(issues({ timeoutSecs: "" })).toEqual({});
    expect(issues({ timeoutSecs: "30" })).toEqual({});
    expect(issues({ timeoutSecs: "0" })).toHaveProperty("timeoutSecs");
    expect(issues({ timeoutSecs: "3601" })).toHaveProperty("timeoutSecs");
    expect(issues({ timeoutSecs: "ten" })).toHaveProperty("timeoutSecs");
  });

  it("checks the connection string in uri mode and ignores the host fields", () => {
    expect(issues({ mode: "uri", uri: "mongodb+srv://c.example.net/?x=1", host: "", port: 0 })).toEqual({});
    expect(issues({ mode: "uri", uri: "mongodb://user:pw@h1,h2/db" })).toEqual({});
    expect(issues({ mode: "uri", uri: "localhost:27017" })).toHaveProperty("uri");
    expect(issues({ mode: "uri", uri: "mongodb://" })).toHaveProperty("uri");
  });

  it("requires username and password with authentication", () => {
    expect(issues({ useAuth: true })).toEqual({ username: "Username is required", password: "Password is required" });
    expect(issues({ useAuth: true, username: "ada", password: "pw" })).toEqual({});
  });
});

describe("connectionFromFormValues", () => {
  it("builds a host target with options", () => {
    const connection = connectionFromFormValues(
      values({ host: " db.local ", port: 27018, tls: true, tlsCaFile: " /ca.pem ", tlsAllowInvalidCertificates: true, replicaSet: "rs0", timeoutSecs: "5" }),
    );

    expect(connection).toEqual({
      name: "Local",
      connector: ConnectorKind.MongoDb,
      target: { kind: "Host", host: "db.local", port: 27018 },
      authentication: { kind: AuthenticationKind.NONE, username: null, password: null, database: null },
      options: { tls: true, tlsCaFile: "/ca.pem", tlsAllowInvalidCertificates: true, replicaSet: "rs0", serverSelectionTimeoutSecs: 5 },
    });
  });

  it("drops tls settings when tls is off and empty fields become null", () => {
    const connection = connectionFromFormValues(values({ tls: false, tlsCaFile: "/ca.pem", tlsAllowInvalidCertificates: true, replicaSet: " " }));
    expect(connection.options).toEqual(defaultConnectionOptions());
  });

  it("builds a uri target without the host options", () => {
    const connection = connectionFromFormValues(values({ mode: "uri", uri: " mongodb+srv://c.example.net ", tls: true, replicaSet: "rs0" }));
    expect(connection.target).toEqual({ kind: "Uri", uri: "mongodb+srv://c.example.net" });
    expect(connection.options).toEqual(defaultConnectionOptions());
  });

  it("leaves the authentication database empty by default, so a string's authSource counts", () => {
    const connection = connectionFromFormValues(values({ mode: "uri", uri: "mongodb://h/?authSource=shop", useAuth: true, username: "u", password: "p" }));
    expect(connection.authentication.database).toBeNull();
  });

  it("sends credentials only with authentication", () => {
    expect(connectionFromFormValues(values({ useAuth: true, username: " ada ", password: "pw", authDatabase: "" })).authentication).toEqual({
      kind: AuthenticationKind.BASIC,
      username: "ada",
      password: "pw",
      database: null,
    });
    expect(connectionFromFormValues(values({ useAuth: false, username: "ada", password: "pw" })).authentication.password).toBeNull();
  });
});

describe("formValuesFromConnection", () => {
  const saved: Connection = {
    id: "c1",
    name: "Atlas",
    connector: ConnectorKind.MongoDb,
    target: { kind: "Uri", uri: "mongodb+srv://c.example.net" },
    authentication: { kind: AuthenticationKind.BASIC, username: "ada", password: null, database: null },
    options: defaultConnectionOptions(),
  };

  it("fills the uri mode and the loaded password", () => {
    const form = formValuesFromConnection(saved, "pw");
    expect(form).toMatchObject({ name: "Atlas", mode: "uri", uri: "mongodb+srv://c.example.net", useAuth: true, username: "ada", password: "pw", authDatabase: "" });
  });

  it("round-trips a host connection with options", () => {
    const host = connectionFromFormValues(values({ host: "h", port: 1, useAuth: true, username: "u", password: "p", authDatabase: "shop", tls: true, timeoutSecs: "9" }));
    const form = formValuesFromConnection({ ...host, id: "x" }, "p");
    expect(connectionFromFormValues(form)).toEqual(host);
  });
});
