import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { effectScope, nextTick, ref } from "vue";
import { useElapsedSeconds } from "../useElapsedSeconds";

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

function setup<T>(initial: T) {
  const key = ref<T>(initial);
  const scope = effectScope();
  const seconds = scope.run(() => useElapsedSeconds(key))!;
  return { key, scope, seconds };
}

describe("useElapsedSeconds", () => {
  it("counts whole seconds while the key is set", () => {
    const { seconds } = setup<number | null>(1);
    expect(seconds.value).toBe(0);

    vi.advanceTimersByTime(999);
    expect(seconds.value).toBe(0);
    vi.advanceTimersByTime(1);
    expect(seconds.value).toBe(1);
    vi.advanceTimersByTime(11_500);
    expect(seconds.value).toBe(12);
  });

  it("starts again from 0 when the key changes", async () => {
    const { key, seconds } = setup<number | null>(1);
    vi.advanceTimersByTime(3_000);
    expect(seconds.value).toBe(3);

    key.value = 2;
    await nextTick();
    expect(seconds.value).toBe(0);
    vi.advanceTimersByTime(2_000);
    expect(seconds.value).toBe(2);
  });

  it("stays at 0 while the key is null or undefined", async () => {
    const { key, seconds } = setup<number | null | undefined>(null);
    vi.advanceTimersByTime(5_000);
    expect(seconds.value).toBe(0);

    key.value = 1;
    await nextTick();
    vi.advanceTimersByTime(2_000);
    key.value = undefined;
    await nextTick();
    expect(seconds.value).toBe(0);
    vi.advanceTimersByTime(5_000);
    expect(seconds.value).toBe(0);
  });

  it("follows the clock when the interval runs late", () => {
    const { seconds } = setup<number | null>(1);
    // the clock jumps ahead without the interval firing in between
    vi.setSystemTime(Date.now() + 7_000);
    vi.advanceTimersByTime(250);
    expect(seconds.value).toBe(7);
  });

  it("stops its timer when the scope ends", () => {
    const { scope, seconds } = setup<number | null>(1);
    scope.stop();
    expect(vi.getTimerCount()).toBe(0);
    vi.advanceTimersByTime(5_000);
    expect(seconds.value).toBe(0);
  });
});
