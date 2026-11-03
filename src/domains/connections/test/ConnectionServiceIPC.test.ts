import { mockIPC } from "@tauri-apps/api/mocks";
import { describe, expect, it } from "vitest";
import { ConnectionServiceIPC } from "../services";

type Call = { cmd: string; args: Record<string, unknown> };

/** Records every IPC call and answers start_* with a task id and await_task_result with `result`. */
function recordIPC(result: unknown = []) {
  const calls: Call[] = [];
  mockIPC((cmd, args) => {
    calls.push({ cmd, args: (args ?? {}) as Record<string, unknown> });
    if (cmd.startsWith("start_")) return "task-1";
    if (cmd === "await_task_result") return result;
    return undefined;
  });
  return calls;
}

describe("ConnectionServiceIPC", () => {
  const service = new ConnectionServiceIPC();

  it("connects by id as a task", async () => {
    const calls = recordIPC(["admin", "shop"]);

    await expect(service.connect("c1")).resolves.toEqual(["admin", "shop"]);

    expect(calls.map((c) => c.cmd)).toEqual(["start_connect", "await_task_result"]);
    expect(calls[0].args).toEqual({ connectionId: "c1" });
    expect(calls[1].args).toEqual({ id: "task-1" });
  });

  it("disconnects by id", async () => {
    const calls = recordIPC();
    await expect(service.disconnect("c1")).resolves.toEqual({ data: undefined, error: null });
    expect(calls).toEqual([{ cmd: "disconnect", args: { connectionId: "c1" } }]);
  });

  it("lists collections by connection id and database", async () => {
    const calls: Call[] = [];
    mockIPC((cmd, args) => {
      calls.push({ cmd, args: args as Record<string, unknown> });
      return ["users"];
    });

    await expect(service.listCollections("c1", "shop")).resolves.toEqual({ data: ["users"], error: null });
    expect(calls).toEqual([{ cmd: "list_collections", args: { connectionId: "c1", databaseName: "shop" } }]);
  });

  it("returns a failed call as error instead of throwing", async () => {
    mockIPC(() => {
      throw { kind: "ConnectionNotFound", message: "x", user_message: "This connection no longer exists." };
    });
    const result = await service.listCollections("gone", "shop");
    expect(result.data).toBeNull();
    expect(result.error).toMatchObject({ kind: "ConnectionNotFound" });
  });

  it("finds documents by connection id as a task", async () => {
    const calls = recordIPC({ documents: [], totalCount: 0 });

    await service.findDocuments("c1", "shop", "users", { a: 1 }, 10, 50);

    expect(calls[0]).toEqual({
      cmd: "start_find_documents",
      args: { connectionId: "c1", databaseName: "shop", collectionName: "users", filter: { a: 1 }, skip: 10, limit: 50 },
    });
  });

  it("tests unsaved form data with an empty id", async () => {
    const calls = recordIPC([]);
    const { id: _id, ...connection } = {
      id: "ignored",
      name: "n",
      connector: "MongoDb" as never,
      target: { kind: "Host" as const, host: "h", port: 1 },
      authentication: { kind: "NONE" as never, username: null, password: null, database: null },
      options: { tls: false, tlsCaFile: null, tlsAllowInvalidCertificates: false, replicaSet: null, serverSelectionTimeoutSecs: null },
    };

    await service.testConnection(connection, "t1");

    expect(calls[0].cmd).toBe("start_test_connection");
    expect((calls[0].args.connection as { id: string }).id).toBe("");
    expect(calls[0].args.testId).toBe("t1");
  });
});
