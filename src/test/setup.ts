import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, beforeEach } from "vitest";

// the logger and stores call Tauri; tests that care about a command override this with their own mockIPC
beforeEach(() => {
  mockIPC(() => undefined);
});

afterEach(() => {
  clearMocks();
});
