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
    name: "Disconnect the connection of the active tab",
    execute: () => {
      const connectionId = deps.tabsStore.getActiveTab()?.context.connectionId;
      if (connectionId) deps.connectionStore.disconnect(connectionId);
    },
  });

  commandRegistry.registerCommand({
    id: Commands.EVENT_EXECUTE,
    name: "Send execute command to components",
    execute: () => {},
  });

}
