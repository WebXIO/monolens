import { onScopeDispose, ref, watch, type WatchSource } from 'vue';

/**
 * Whole seconds since `key` last changed. Counts while the key is set, and is 0 while it is
 * null or undefined, so a running step can show how long it has been waiting.
 */
export function useElapsedSeconds(key: WatchSource<unknown>) {
  const seconds = ref(0);
  let timer: ReturnType<typeof setInterval> | undefined;

  function stop() {
    if (timer !== undefined) clearInterval(timer);
    timer = undefined;
  }

  watch(
    key,
    (value) => {
      stop();
      seconds.value = 0;
      if (value === null || value === undefined) return;

      // measured against the clock, as a busy page may run the interval late
      const startedAt = Date.now();
      timer = setInterval(() => {
        seconds.value = Math.floor((Date.now() - startedAt) / 1000);
      }, 250);
    },
    { immediate: true },
  );

  onScopeDispose(stop);

  return seconds;
}
