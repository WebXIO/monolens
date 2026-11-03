import { describe, expect, it, vi } from "vitest";
import { DefaultBindings, initCommands } from "../config";
import { Commands } from "../models";
import { CommandRegistry } from "../registry";

function registryWith(execute = vi.fn()) {
  const registry = new CommandRegistry();
  registry.registerCommand({ id: Commands.TAB_CLOSE, name: "close", execute });
  return { registry, execute };
}

describe("CommandRegistry", () => {
  it("runs a registered command and notifies its subscribers", () => {
    const { registry, execute } = registryWith();
    const subscriber = vi.fn();
    registry.subscribe(Commands.TAB_CLOSE, subscriber);

    registry.executeCommand(Commands.TAB_CLOSE);

    expect(execute).toHaveBeenCalledOnce();
    expect(subscriber).toHaveBeenCalledOnce();
  });

  it("ignores an unknown command", () => {
    const { registry, execute } = registryWith();
    expect(() => registry.executeCommand("nope")).not.toThrow();
    expect(execute).not.toHaveBeenCalled();
  });

  it("unsubscribes exactly the given subscriber without calling it", () => {
    const { registry } = registryWith();
    const first = vi.fn();
    const second = vi.fn();
    registry.subscribe(Commands.TAB_CLOSE, first);
    registry.subscribe(Commands.TAB_CLOSE, second);

    registry.unsubscribe(Commands.TAB_CLOSE, second);
    expect(first).not.toHaveBeenCalled();
    expect(second).not.toHaveBeenCalled();

    registry.executeCommand(Commands.TAB_CLOSE);
    expect(first).toHaveBeenCalledOnce();
    expect(second).not.toHaveBeenCalled();
  });

  it("does nothing when unsubscribing an unknown subscriber or event", () => {
    const { registry } = registryWith();
    const subscriber = vi.fn();
    registry.subscribe(Commands.TAB_CLOSE, subscriber);

    registry.unsubscribe(Commands.TAB_CLOSE, vi.fn());
    registry.unsubscribe(Commands.EVENT_EXECUTE, subscriber);

    registry.executeCommand(Commands.TAB_CLOSE);
    expect(subscriber).toHaveBeenCalledOnce();
  });

  it("runs nothing while disabled", () => {
    const { registry, execute } = registryWith();
    registry.disableExecution();
    registry.executeCommand(Commands.TAB_CLOSE);
    expect(execute).not.toHaveBeenCalled();

    registry.enableExecution();
    registry.executeCommand(Commands.TAB_CLOSE);
    expect(execute).toHaveBeenCalledOnce();
  });
});

describe("default bindings", () => {
  it("bind every command that initCommands registers, with the mod key", () => {
    const registry = new CommandRegistry();
    initCommands(registry, { tabsStore: {}, connectionStore: {} });

    for (const binding of DefaultBindings) {
      expect(registry.commands.has(binding.commandId)).toBe(true);
      expect(binding.keyCode.startsWith("mod+")).toBe(true);
    }
    expect(DefaultBindings.map((b) => b.keyCode)).toEqual(["mod+w", "mod+shift+d", "mod+enter"]);
  });
});
