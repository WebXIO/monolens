<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import { SidebarProps, SidebarContent, SidebarGroup, Sidebar } from './ui/sidebar';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { Button } from "@/components/ui/button"
import { ChevronDown, Database, RefreshCcw, Loader2, ChevronUp, FileJson, LogOut } from 'lucide-vue-next';
import { ScrollArea } from '@/components/ui/scroll-area';
import { useConnectionStore } from '@/stores/connectionStore';
import { invoke } from '@tauri-apps/api/core';
import { tryCatch } from '@/utils/result';
import { useLogger } from '@/composables/useLogger';
import { useTabsStore } from '@/stores/tabsStore';
import { Context } from '@/domains/context';
import { TabKind } from '@/domains/tabs';

const props = defineProps<SidebarProps>()

const logger = useLogger("AppSidebar");
const store = useConnectionStore();
const tabsStore = useTabsStore();

const toggledDatabases = ref<Set<string>>(new Set());
const collections = ref<Record<string, string[]>>({});

async function toggleDatabase(dbName: string) {
   if (!store.activeConnection) return;

   if (toggledDatabases.value.has(dbName)) {
      toggledDatabases.value.delete(dbName);
      return;
   }

   if (collections.value[dbName]) {
      toggledDatabases.value.add(dbName);
      return;
   }

   const { data, error } = await tryCatch<string[]>(invoke('list_collections', { connection: store.activeConnection, databaseName: dbName }))

   if (data) {
      collections.value[dbName] = data;
      toggledDatabases.value.add(dbName);
   }
   else
      logger.error(error)

}

function isDatabaseExpended(name: string) {
   return toggledDatabases.value.has(name);
}

function handleCollectionSelect(database: string, collection: string) {
   if(!store.activeConnection) return;

   const context: Context = {
      collection: collection,
      database: database,
      connection: store.activeConnection
   }

   let tabId = tabsStore.hasContext(context, TabKind.QUERY);

   if (tabId === null) {
      tabId = tabsStore.addTab(context, TabKind.QUERY);
   }

   tabsStore.switchTab(tabId);
}

async function refreshDatabases() {
   if (store.activeConnection) {
      await store.connectTo(store.activeConnection);
   }
}

function disconnect() {
   store.disconnect();
   toggledDatabases.value.clear();
   collections.value = {};
}

const activeTabContext = computed(() => tabsStore.getActiveTab()?.context ?? null);

function isCollectionActive(database: string, collection: string): boolean {
   if (!activeTabContext.value || !store.activeConnection) return false;
   return (
      activeTabContext.value.database === database &&
      activeTabContext.value.collection === collection &&
      activeTabContext.value.connection.id === store.activeConnection.id
   );
}

watch(
   () => tabsStore.activeTab,
   async (newActiveTabId) => {
      if (!newActiveTabId || !store.activeConnection) return;

      const activeTab = tabsStore.getActiveTab();
      if (!activeTab) return;

      const { database } = activeTab.context;

      if (!toggledDatabases.value.has(database)) {
         await toggleDatabase(database);
      }
   }
);
</script>

<template>
   <Sidebar class="top-(--header-height) h-[calc(100svh-var(--header-height))]!" v-bind="props">
      <SidebarContent>
         <SidebarGroup class="flex flex-row item-center items-center-safe justify-between border-b border-border">
            <span className="text-xs font-medium uppercase tracking-wider text-muted-foreground">
               Explorer
            </span>
            <div class="flex gap-0.5">
               <TooltipProvider>
                  <Tooltip>
                     <TooltipTrigger as-child>
                        <Button variant="ghost" size="icon" class="h-6 w-6" :disabled="store.isDatabasesLoading"
                           @click="refreshDatabases">
                           <Loader2 v-if="store.isDatabasesLoading" class="h-3.5 w-3.5 animate-spin" />
                           <RefreshCcw v-else class="h-3.5 w-3.5" />
                        </Button>
                     </TooltipTrigger>
                     <TooltipContent side="bottom" class="text-xs">
                        Refresh
                     </TooltipContent>
                  </Tooltip>
               </TooltipProvider>
               <TooltipProvider>
                  <Tooltip>
                     <TooltipTrigger as-child>
                        <Button
                           variant="ghost"
                           size="icon"
                           class="h-6 w-6 hover:bg-red-600 hover:text-white"
                           :disabled="store.isDatabasesLoading"
                           @click="disconnect"
                        >
                           <Loader2 v-if="store.isDatabasesLoading" class="h-3.5 w-3.5 animate-spin" />
                           <LogOut v-else class="h-3.5 w-3.5" />
                        </Button>
                     </TooltipTrigger>
                     <TooltipContent side="bottom" class="text-xs">
                        Disconnect from Database
                     </TooltipContent>
                  </Tooltip>
               </TooltipProvider>
            </div>
         </SidebarGroup>
         <SidebarGroup>
            <ScrollArea class="flex-1">
               <div v-if="store.isDatabasesLoading" class="flex items-center gap-2 px-2 py-4 text-sm text-muted-foreground">
                  <Loader2 class="h-4 w-4 animate-spin" />
                  <span>Loading databases...</span>
               </div>

               <div v-else-if="store.databases.length === 0" class="px-2 py-4 text-sm text-muted-foreground">
                  No databases found
               </div>

               <div v-else v-for="database in store.databases" :key="database">
                  <div
                     class="group flex cursor-pointer items-center gap-1 rounded-md px-2 py-1.5 text-sm transition-colors hover:bg-secondary"
                     @click="toggleDatabase(database)">
                     <Button variant="ghost" size="icon" class="h-4 w-4 hover:bg-transparent">
                        <ChevronDown v-if="!isDatabaseExpended(database)" />
                        <ChevronUp v-else />
                     </Button>
                     <Database class="h-4 w-4" />
                     <span>{{ database }}</span>
                  </div>
                  <template v-if="isDatabaseExpended(database)">
                     <div v-for="collection in collections[database]" class="ps-8">

                        <div
                        class="group flex cursor-pointer items-center gap-1 rounded-md px-2 py-1.5 text-sm transition-colors hover:bg-secondary"
                        :class="{ 'bg-primary/15 text-primary font-medium': isCollectionActive(database, collection) }"
                        @click="handleCollectionSelect(database, collection)"
                        >
                        <FileJson class="h-4 w-4" />
                        <span>{{ collection }}</span>
                     </div>
                  </div>
                  </template>
               </div>
            </ScrollArea>
         </SidebarGroup>
      </SidebarContent>
   </Sidebar>
</template>