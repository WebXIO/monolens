import { useLogger } from "@/composables/useLogger";
import { useDomain } from "@/domains";
import { CommandRegistry, eventToKeyString } from "@/domains/shortcuts";
import { DefaultBindings, initCommands } from "@/domains/shortcuts/config";
import { useConnectionStore } from "@/stores/connectionStore";
import { useTabsStore } from "@/stores/tabsStore";
import { App } from "vue";

const logger = useLogger("ShortcutPlugin");

export const shortcuts = {
  install(app: App) {
    const commandRegistry = useDomain<CommandRegistry>(CommandRegistry, app);
    const tabsStore = useTabsStore();
    const connectionStore = useConnectionStore();

    initCommands(commandRegistry, {
      connectionStore: connectionStore,
      tabsStore: tabsStore,
    });

    function handleKeyDown(ev: KeyboardEvent) {
      const keyString = eventToKeyString(ev);
      const binding = DefaultBindings.find((b) => b.keyCode === keyString);

      if (binding) {
        logger.debug("Found binding: ", binding);
        ev.preventDefault();
        commandRegistry.executeCommand(binding.commandId);
      }
    }

    globalThis.addEventListener("keydown", handleKeyDown);
  },
};
