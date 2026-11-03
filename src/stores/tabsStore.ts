import { useLogger } from "@/composables/useLogger";
import { Context } from "@/domains/context";
import { Tab, TabKind } from "@/domains/tabs";
import { defineStore } from "pinia";
import { ref } from "vue";

export const useTabsStore = defineStore("tabs", () => {
  const logger = useLogger("TabsStore");
  const tabs = ref<Map<string, Tab>>(new Map());
  const activeTab = ref<string | null>(null);
  const tabHistory = ref<string[]>([]);

  function addTab(context: Context, type: TabKind) {
    const id = globalThis.crypto.randomUUID();

    tabs.value.set(id, {
      id,
      context,
      type,
    });

    return id;
  }

  function switchTab(id: string) {
    const tab = tabs.value.get(id);

    if (!tab) {
      logger.warn(`Could not get tab with id ${id}`);
      return;
    }

    const idx = tabHistory.value.indexOf(id);

    if (idx !== -1) {
      tabHistory.value.splice(idx, 1);
    }

    tabHistory.value.push(id);

    activeTab.value = id;
  }

  function deleteTab(id: string) {
    if (!tabs.value.has(id)) {
      logger.warn(`Could not find tab with id ${id}`);
      return;
    }

    tabs.value.delete(id);

    const idx = tabHistory.value.indexOf(id);
    if (idx !== -1) {
      tabHistory.value.splice(idx, 1);
    }

    if (activeTab.value === id) {
      switchToLastTab();
    }
  }

  function switchToLastTab() {
    const lastTab =
      tabHistory.value.length > 0
        ? tabHistory.value[tabHistory.value.length - 1]
        : null;

    if (lastTab) switchTab(lastTab);
    else activeTab.value = null;
  }

  function getActiveTab() {
    return activeTab.value ? (tabs.value.get(activeTab.value) ?? null) : null;
  }

  function hasContext(context: Context, type: TabKind) {
    for (const [id, tab] of tabs.value.entries()) {
      if (
        tab.type === type &&
        tab.context.collection === context.collection &&
        tab.context.database === context.database &&
        tab.context.connectionId === context.connectionId
      )
        return id;
    }

    return null;
  }

  /** Closes every tab of a connection, e.g. when it is disconnected. */
  function closeTabsOf(connectionId: string) {
    for (const [id, tab] of [...tabs.value.entries()]) {
      if (tab.context.connectionId === connectionId) deleteTab(id);
    }
  }

  function closeActiveTab() {
    logger.debug(activeTab.value);
    if (!activeTab.value) return;

    deleteTab(activeTab.value);
  }

  return {
    tabs,
    activeTab,
    addTab,
    switchTab,
    deleteTab,
    getActiveTab,
    hasContext,
    closeTabsOf,
    closeActiveTab,
  };
});
