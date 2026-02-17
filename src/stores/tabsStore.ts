import { useLogger } from "@/composables/useLogger";
import { Context } from "@/domains/context";
import { Tab, TabKind } from "@/domains/tabs";
import { defineStore } from "pinia";
import { ref } from "vue";

export const useTabsStore = defineStore("tabs", () => {
  const logger = useLogger("TabsStore");
  const tabs = ref<Map<string, Tab>>(new Map());
  const activeTab = ref<string | null>(null);

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

    activeTab.value = id;
  }

  function deleteTab(id: string) {
    if (!tabs.value.has(id)) {
      logger.warn(`Could not find tab with id ${id}`);
      return;
    }

    tabs.value.delete(id);
  }

  function getActiveTab() {
    return activeTab.value ? (tabs.value.get(activeTab.value) ?? null) : null;
  }

  function hasContext(context: Context, type: TabKind) {
   for(const [id, tab] of tabs.value.entries()) {
      if(
         tab.type === type &&
         tab.context.collection === context.collection &&
         tab.context.database === context.database &&
         tab.context.connection.id === context.connection.id
      ) return id
   }

   return null;
  }

  return {
    tabs,
    addTab,
    switchTab,
    deleteTab,
    getActiveTab,
    hasContext
  };
});
