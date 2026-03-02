/**
 * A backend error object coming from Tauri commands.
 * Both DriverError and CommandError serialize with these fields.
 */
interface BackendError {
  user_message?: string;
  message?: string;
  kind?: string;
}

function isBackendError(e: unknown): e is BackendError {
  return typeof e === 'object' && e !== null && ('user_message' in e || 'message' in e);
}

/**
 * Extracts a human-readable error message from any error shape returned by the backend.
 *
 * Priority:
 *  1. `user_message` field (set by DriverError / CommandError in the backend)
 *  2. `message` field (raw technical detail — fallback)
 *  3. Error.message (standard JS Error)
 *  4. Generic fallback string
 */
export function extractErrorMessage(e: unknown): string {
  if (isBackendError(e)) {
    if (e.user_message) return e.user_message;
    if (e.message) return e.message;
  }

  if (e instanceof Error) {
    return e.message;
  }

  if (typeof e === 'string') {
    return e;
  }

  return 'An unexpected error occurred.';
}
