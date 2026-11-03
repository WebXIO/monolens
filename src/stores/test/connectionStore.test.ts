import { mockIPC } from "@tauri-apps/api/mocks";
import { emit } from "@tauri-apps/api/event";
import { createPinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createApp } from "vue";
import { toast } from "vue-sonner";
import { registerDomains } from "@/domains";
import { AuthenticationKind, ConnectorKind, defaultConnectionOptions, type TestStage } from "@/domains/connections";
import { useConnectionStore } from "../connectionStore";

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
