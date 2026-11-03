import { afterEach, describe, expect, it } from "vitest";
import { useConfirm } from "../useConfirm";

afterEach(() => {
  useConfirm().answer(false);
});

describe("useConfirm", () => {
  it("opens with the request and resolves to the answer", async () => {
    const { state, confirm, answer } = useConfirm();

    const pending = confirm({ title: "Delete connection", destructive: true });
    expect(state.open).toBe(true);
    expect(state.request).toEqual({ title: "Delete connection", destructive: true });

    answer(true);
    await expect(pending).resolves.toBe(true);
    expect(state.open).toBe(false);
  });

  it("resolves to false when declined", async () => {
    const { confirm, answer } = useConfirm();
    const pending = confirm({ title: "x" });
    answer(false);
    await expect(pending).resolves.toBe(false);
  });

  it("a new question declines the one still open", async () => {
    const { state, confirm, answer } = useConfirm();
    const first = confirm({ title: "first" });
    const second = confirm({ title: "second" });

    await expect(first).resolves.toBe(false);
    expect(state.request?.title).toBe("second");
    answer(true);
    await expect(second).resolves.toBe(true);
  });

  it("an answer without a question does nothing", () => {
    const { state, answer } = useConfirm();
    expect(() => answer(true)).not.toThrow();
    expect(state.open).toBe(false);
  });
});
