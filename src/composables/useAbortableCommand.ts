import { invoke } from '@tauri-apps/api/core';
import { useLogger } from './useLogger';

export class TaskCancelledError extends Error {
  constructor() {
    super('Task was cancelled');
    this.name = 'TaskCancelledError';
  }
}

export async function abortableInvoke<T>(
  startCommand: string,
  args: Record<string, unknown>,
  options?: { signal?: AbortSignal },
): Promise<T> {
  const logger = useLogger("abortableInvoke");
  const signal = options?.signal;

  if (signal?.aborted) {
    throw new TaskCancelledError();
  }

  const id = await invoke<string>(startCommand, args);

  const abortHandler = () => {
    invoke('cancel_task', { id }).catch(() => {
        logger.error(`Failed to cancel task with id ${id}`);
    });
  };

  signal?.addEventListener('abort', abortHandler);

  try {
    const result = await invoke<T>('await_task_result', { id });
    return result;
  } catch (error: unknown) {
    if (
      signal?.aborted ||
      (typeof error === 'object' &&
        error !== null &&
        'kind' in error &&
        (error as Record<string, unknown>).kind === 'Cancelled')
    ) {
      throw new TaskCancelledError();
    }

    if (
      typeof error === 'object' &&
      error !== null &&
      'kind' in error &&
      (error as Record<string, unknown>).kind === 'Failed'
    ) {
      // The message field contains the stringified JSON of the original error
      const message = (error as Record<string, unknown>).message;
      if (typeof message === 'string') {
        try {
          throw JSON.parse(message);
        } catch (parseError) {
          if (parseError === message) {
            throw error;
          }
          throw parseError;
        }
      }
    }

    throw error;
  } finally {
    signal?.removeEventListener('abort', abortHandler);
  }
}
