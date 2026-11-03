import { describe, expect, it } from "vitest";
import { defaultConnectionOptions, describeTarget } from "../models";

describe("describeTarget", () => {
  it("shows host and port", () => {
    expect(describeTarget({ kind: "Host", host: "localhost", port: 27017 })).toBe("localhost:27017");
  });

  it("shows a connection string as it is stored", () => {
    expect(describeTarget({ kind: "Uri", uri: "mongodb+srv://c.example.net/?retryWrites=true" })).toBe(
      "mongodb+srv://c.example.net/?retryWrites=true",
    );
  });
});

describe("defaultConnectionOptions", () => {
  it("has no tls, replica set or timeout", () => {
    expect(defaultConnectionOptions()).toEqual({
      tls: false,
      tlsCaFile: null,
      tlsAllowInvalidCertificates: false,
      replicaSet: null,
      serverSelectionTimeoutSecs: null,
    });
  });

  it("returns a new object every time", () => {
    expect(defaultConnectionOptions()).not.toBe(defaultConnectionOptions());
  });
});
