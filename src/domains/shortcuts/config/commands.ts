import { Commands } from "../models";
import { CommandRegistry } from "../registry";

export function initCommands(
  commandRegistry: CommandRegistry,
  deps: { tabsStore: any; connectionStore: any },
) {
  commandRegistry.registerCommand({
    id: Commands.TAB_CLOSE,
    name: "Close current active Tab",
    execute: () => deps.tabsStore.closeActiveTab(),
  });

  commandRegistry.registerCommand({
    id: Commands.CONNECTION_DISCONNECT,
    name: "Disconnect from current connection",
    execute: () => deps.connectionStore.disconnect(),
  });

  commandRegistry.registerCommand({
    id: Commands.EVENT_EXECUTE,
    name: "Send execute command to components",
    execute: () => {},
  });

}
