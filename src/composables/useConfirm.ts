import { reactive, readonly } from 'vue';

export type ConfirmRequest = {
  title: string;
  description?: string;
  /** default "Confirm" */
  confirmLabel?: string;
  /** red confirm button, for deleting */
  destructive?: boolean;
};

// one dialog for the whole app, mounted in App.vue
const state = reactive<{ open: boolean; request: ConfirmRequest | null }>({ open: false, request: null });
let resolveAnswer: ((confirmed: boolean) => void) | null = null;

function answer(confirmed: boolean) {
  state.open = false;
  const resolve = resolveAnswer;
  resolveAnswer = null;
  resolve?.(confirmed);
}

/** Asks with the app's dialog instead of the browser's confirm(); resolves to the answer. */
function confirm(request: ConfirmRequest): Promise<boolean> {
  // a new question cancels one that is still open
  answer(false);
  state.request = request;
  state.open = true;
  return new Promise((resolve) => {
    resolveAnswer = resolve;
  });
}

export function useConfirm() {
  return { state: readonly(state), confirm, answer };
}
