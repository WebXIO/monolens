import { mockIPC } from "@tauri-apps/api/mocks";
import { emit } from "@tauri-apps/api/event";
import { createPinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createApp } from "vue";
import { toast } from "vue-sonner";
import { registerDomains } from "@/domains";
import { AuthenticationKind, ConnectorKind, defaultConnectionOptions, type TestStage } from "@/domains/connections";
import { TabKind } from "@/domains/tabs";
import { useConnectionStore } from "../connectionStore";
import { useTabsStore } from "../tabsStore";

vi.mock("vue-sonner", () => ({ toast: { error: vi.fn(), info: vi.fn() } }));

/** The store runs inside an app, because it reads its service through inject. */
function setupStore() {
  const app = createApp({});
  registerDomains(app);
  app.use(createPinia());
  return app.runWithContext(() => useConnectionStore());
}

const form = {
  name: "Local",
  connector: ConnectorKind.MongoDb,
  target: { kind: "Host" as const, host: "localhost", port: 27017 },
  authentication: { kind: AuthenticationKind.NONE, username: null, password: null, database: null },
  options: defaultConnectionOptions(),
};

const stage = (status: TestStage["status"], title: string, detail: string | null = null): TestStage => ({ status, title, detail });

type Handler = (cmd: string, args: Record<string, any>) => unknown;

function mockBackend(handler: Handler) {
  mockIPC(async (cmd, args) => handler(cmd, (args ?? {}) as Record<string, any>), { shouldMockEvents: true });
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("connectionStore.testConnection", () => {
  it("builds the stages from the progress events of its own test only", async () => {
    const seen: TestStage[][] = [];
    mockBackend(async (cmd, args) => {
      if (cmd === "start_test_connection") {
        await emit("test-connection-progress", { testId: args.testId, stageIndex: 0, stage: stage("pending", "Initialize") });
        await emit("test-connection-progress", { testId: args.testId, stageIndex: 1, stage: stage("pending", "Ping Database") });
        await emit("test-connection-progress", { testId: "another-test", stageIndex: 0, stage: stage("failed", "Other") });
        await emit("test-connection-progress", { testId: args.testId, stageIndex: 0, stage: stage("passed", "Initialize") });
        return "task-1";
      }
      if (cmd === "await_task_result") {
        seen.push([...store.testStages]);
        return [stage("passed", "Initialize"), stage("skipped", "Ping Database", "No permission")];
      }
    });
    const store = setupStore();

    const result = await store.testConnection(form);

    expect(seen[0]).toEqual([stage("passed", "Initialize"), stage("pending", "Ping Database")]);
    expect(result).toEqual([stage("passed", "Initialize"), stage("skipped", "Ping Database", "No permission")]);
    expect(store.testStages).toEqual(result);
    expect(store.isTesting).toBe(false);
  });

  it("sends a new test id with every test", async () => {
    const ids: string[] = [];
    mockBackend((cmd, args) => {
      if (cmd === "start_test_connection") ids.push(args.testId);
      if (cmd === "start_test_connection") return "task";
      if (cmd === "await_task_result") return [];
    });
    const store = setupStore();

    await store.testConnection(form);
    await store.testConnection(form);

    expect(ids).toHaveLength(2);
    expect(ids[0]).not.toBe(ids[1]);
  });

  it("keeps the reported stages and shows the reason when the test fails", async () => {
    mockBackend(async (cmd, args) => {
      if (cmd === "start_test_connection") {
        await emit("test-connection-progress", { testId: args.testId, stageIndex: 0, stage: stage("failed", "Ping Database") });
        return "task-1";
      }
      if (cmd === "await_task_result") {
        throw { kind: "Failed", message: JSON.stringify({ kind: "PingFailed", message: "x", user_message: "Authentication failed. Check your username." }), user_message: "failed" };
      }
    });
    const store = setupStore();

    const result = await store.testConnection(form);

    expect(result).toEqual([]);
    expect(store.testStages).toEqual([stage("failed", "Ping Database")]);
    expect(toast.error).toHaveBeenCalledWith("Authentication failed. Check your username.");
    expect(store.isTesting).toBe(false);
  });

  it("reports a cancelled test without an error", async () => {
    mockBackend((cmd) => {
      if (cmd === "start_test_connection") return "task-1";
      if (cmd === "await_task_result") throw { kind: "Cancelled", message: "Task was cancelled", user_message: "cancelled" };
    });
    const store = setupStore();

    await store.testConnection(form);

    expect(toast.info).toHaveBeenCalledWith("Test cancelled.");
    expect(toast.error).not.toHaveBeenCalled();
  });
});

describe("connectionStore with several connections", () => {
  const saved = (id: string) => ({ ...form, id, name: `Server ${id}` });

  /** Answers start_connect/await_task_result with databases named after the connection. */
  function mockServers(fail: Record<string, unknown> = {}, stored: unknown[] = []) {
    const calls: { cmd: string; args: Record<string, any> }[] = [];
    let collectionsCalls = 0;
    mockBackend((cmd, args) => {
      calls.push({ cmd, args });
      if (cmd === "get_connections") return stored;
      if (cmd === "start_connect") return `task:${args.connectionId}`;
      if (cmd === "await_task_result") {
        const connectionId = String(args.id).replace("task:", "");
        if (fail[connectionId]) throw fail[connectionId];
        return [`db-${connectionId}`];
      }
      if (cmd === "list_collections") {
        collectionsCalls++;
        return [`${args.databaseName}-collection-${collectionsCalls}`];
      }
      return undefined;
    });
    return calls;
  }

  async function storeWith(...ids: string[]) {
    const store = setupStore();
    store.connections = ids.map(saved);
    return store;
  }

  it("opens two connections side by side, in order", async () => {
    mockServers();
    const store = await storeWith("a", "b");

    await store.connectTo(saved("a"));
    await store.connectTo(saved("b"));

    expect(store.openConnections.map((c) => c.id)).toEqual(["a", "b"]);
    expect(store.sessionOf("a")?.databases).toEqual(["db-a"]);
    expect(store.sessionOf("b")?.databases).toEqual(["db-b"]);
    expect(store.hasOpenConnections).toBe(true);
  });

  it("counts as open only once connected", async () => {
    let release!: () => void;
    mockBackend((cmd) => {
      if (cmd === "start_connect") return "task";
      if (cmd === "await_task_result") return new Promise((resolve) => (release = () => resolve(["db"])));
    });
    const store = await storeWith("a");

    const connecting = store.connectTo(saved("a"));
    await vi.waitFor(() => expect(release).toBeTypeOf("function"));
    expect(store.isConnecting("a")).toBe(true);
    expect(store.hasOpenConnections).toBe(false);

    release();
    await connecting;
    expect(store.isConnected("a")).toBe(true);
    expect(store.hasOpenConnections).toBe(true);
  });

  it("does not connect an open connection again", async () => {
    const calls = mockServers();
    const store = await storeWith("a");

    await store.connectTo(saved("a"));
    await store.connectTo(saved("a"));

    expect(calls.filter((c) => c.cmd === "start_connect")).toHaveLength(1);
  });

  it("drops a connection that fails to connect and shows why", async () => {
    mockServers({ b: { kind: "Failed", message: JSON.stringify({ kind: "PingFailed", message: "x", user_message: "Could not reach the database." }), user_message: "failed" } });
    const store = await storeWith("a", "b");
    await store.connectTo(saved("a"));

    expect(await store.connectTo(saved("b"))).toBe(false);

    expect(store.sessionOf("b")).toBeNull();
    expect(store.openConnections.map((c) => c.id)).toEqual(["a"]);
    expect(toast.error).toHaveBeenCalledWith("Could not reach the database.");
  });

  it("drops a cancelled connection without an error", async () => {
    mockServers({ a: { kind: "Cancelled", message: "Task was cancelled", user_message: "cancelled" } });
    const store = await storeWith("a");

    expect(await store.connectTo(saved("a"))).toBe(false);

    expect(store.sessionOf("a")).toBeNull();
    expect(toast.error).not.toHaveBeenCalled();
  });

  it("disconnecting closes only that connection's tabs and tells the rust core", async () => {
    const calls = mockServers();
    const store = await storeWith("a", "b");
    await store.connectTo(saved("a"));
    await store.connectTo(saved("b"));
    const tabs = createApp({}).runWithContext(() => useTabsStore());
    const keep = tabs.addTab({ connectionId: "b", database: "db-b", collection: "users" }, TabKind.QUERY);
    tabs.switchTab(tabs.addTab({ connectionId: "a", database: "db-a", collection: "users" }, TabKind.QUERY));

    await store.disconnect("a");

    expect(store.openConnections.map((c) => c.id)).toEqual(["b"]);
    expect([...tabs.tabs.keys()]).toEqual([keep]);
    expect(calls).toContainEqual({ cmd: "disconnect", args: { connectionId: "a" } });
  });

  it("disconnects an open connection before deleting it", async () => {
    const calls = mockServers();
    const store = await storeWith("a");
    await store.connectTo(saved("a"));

    expect(await store.deleteConnection("a")).toBe(true);

    const order = calls.map((c) => c.cmd).filter((cmd) => cmd === "disconnect" || cmd === "delete_connection");
    expect(order).toEqual(["disconnect", "delete_connection"]);
    expect(store.sessionOf("a")).toBeNull();
    expect(store.connections).toEqual([]);
  });

  it("caches the collections of a database until the connection is refreshed", async () => {
    const calls = mockServers();
    const store = await storeWith("a");
    await store.connectTo(saved("a"));

    expect(await store.listCollections("a", "db-a")).toEqual(["db-a-collection-1"]);
    expect(await store.listCollections("a", "db-a")).toEqual(["db-a-collection-1"]);

    expect(await store.refresh("a")).toBe(true);
    expect(store.sessionOf("a")?.collections).toEqual({});
    expect(await store.listCollections("a", "db-a")).toEqual(["db-a-collection-2"]);
    expect(calls.filter((c) => c.cmd === "start_connect")).toHaveLength(2);
  });

  it("lists no collections of a connection that isn't open", async () => {
    const calls = mockServers();
    const store = await storeWith("a");
    expect(await store.listCollections("a", "db")).toBeNull();
    expect(calls.filter((c) => c.cmd === "list_collections")).toHaveLength(0);
  });

  it("reloads what the rust core stored after an edit and reconnects an open connection", async () => {
    // the rust core keeps no password and takes credentials out of a connection string
    const calls = mockServers({}, [{ ...saved("a"), name: "Renamed", target: { kind: "Uri", uri: "mongodb://h" } }]);
    const store = await storeWith("a");
    await store.connectTo(saved("a"));
    const edited = { ...saved("a"), name: "Renamed", authentication: { ...form.authentication, kind: AuthenticationKind.BASIC, username: "u", password: "secret" } };

    expect(await store.updateConnection("a", edited)).toBe(true);

    expect(store.getConnection("a")?.name).toBe("Renamed");
    expect(store.getConnection("a")?.target).toEqual({ kind: "Uri", uri: "mongodb://h" });
    expect(calls.filter((c) => c.cmd === "get_connections")).toHaveLength(1);
    expect(calls.filter((c) => c.cmd === "start_connect")).toHaveLength(2);
  });
});
