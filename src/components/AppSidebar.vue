<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { SidebarProps, SidebarContent, SidebarGroup, Sidebar } from './ui/sidebar';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { Button } from "@/components/ui/button"
import { ChevronDown, ChevronRight, Database, FileJson, Loader2, LogOut, Pencil, RefreshCcw, Server, X } from '@lucide/vue';
import { ScrollArea } from '@/components/ui/scroll-area';
import ConnectionDialog from '@/components/connection/ConnectionDialog.vue';
import { useConnectionStore } from '@/stores/connectionStore';
import { useTabsStore } from '@/stores/tabsStore';
import { Context } from '@/domains/context';
import { TabKind } from '@/domains/tabs';
import type { Connection } from '@/domains/connections';

const props = defineProps<SidebarProps>()

const store = useConnectionStore();
const tabsStore = useTabsStore();

// connections are open by default, databases closed
const collapsedConnections = ref<Set<string>>(new Set());
const expandedDatabases = ref<Set<string>>(new Set());

const editOpen = ref(false);
const editingConnection = ref<Connection | undefined>();

function databaseKey(connectionId: string, database: string) {
   return `${connectionId}/${database}`;
}

function toggleConnection(connectionId: string) {
   const next = new Set(collapsedConnections.value);
   if (next.has(connectionId)) next.delete(connectionId);
   else next.add(connectionId);
   collapsedConnections.value = next;
}

function isDatabaseExpanded(connectionId: string, database: string) {
   return expandedDatabases.value.has(databaseKey(connectionId, database));
}

async function expandDatabase(connectionId: string, database: string) {
   const collections = await store.listCollections(connectionId, database);
   if (!collections) return;
   expandedDatabases.value = new Set(expandedDatabases.value).add(databaseKey(connectionId, database));
}

async function toggleDatabase(connectionId: string, database: string) {
   if (isDatabaseExpanded(connectionId, database)) {
      const next = new Set(expandedDatabases.value);
      next.delete(databaseKey(connectionId, database));
      expandedDatabases.value = next;
      return;
   }
   await expandDatabase(connectionId, database);
}

function collectionsOf(connectionId: string, database: string): string[] {
   return store.sessionOf(connectionId)?.collections[database] ?? [];
}

function forgetExpanded(connectionId: string) {
   const prefix = `${connectionId}/`;
   expandedDatabases.value = new Set([...expandedDatabases.value].filter((key) => !key.startsWith(prefix)));
}

function handleCollectionSelect(connectionId: string, database: string, collection: string) {
   const context: Context = { connectionId, database, collection };

   let tabId = tabsStore.hasContext(context, TabKind.QUERY);

   if (tabId === null) {
      tabId = tabsStore.addTab(context, TabKind.QUERY);
   }

   tabsStore.switchTab(tabId);
}

async function refresh(connectionId: string) {
   // collections are listed again when a database is opened
   if (await store.refresh(connectionId)) forgetExpanded(connectionId);
}

function openEdit(connection: Connection) {
   editingConnection.value = connection;
   editOpen.value = true;
}

function disconnect(connectionId: string) {
   forgetExpanded(connectionId);
   store.disconnect(connectionId);
}

const activeTabContext = computed(() => tabsStore.getActiveTab()?.context ?? null);

function isCollectionActive(connectionId: string, database: string, collection: string): boolean {
   const context = activeTabContext.value;
   return context !== null
      && context.connectionId === connectionId
      && context.database === database
      && context.collection === collection;
}

// show the active tab's collection in the tree
watch(
   activeTabContext,
   async (context) => {
      if (!context) return;
      if (collapsedConnections.value.has(context.connectionId)) toggleConnection(context.connectionId);
      if (!isDatabaseExpanded(context.connectionId, context.database)) {
         await expandDatabase(context.connectionId, context.database);
      }
   }
);
</script>

<template>
   <Sidebar class="top-(--header-height) h-[calc(100svh-var(--header-height))]!" v-bind="props">
      <SidebarContent>
         <SidebarGroup class="flex flex-row items-center justify-between border-b border-border">
            <span class="text-xs font-medium uppercase tracking-wider text-muted-foreground">
               Explorer
            </span>
         </SidebarGroup>
         <SidebarGroup>
            <ScrollArea class="flex-1">
               <TooltipProvider :delay-duration="300">
                  <div v-for="connection in store.openConnections" :key="connection.id" class="mb-1">
                     <div class="group flex items-center gap-1 rounded-md px-1 py-1 text-sm hover:bg-secondary">
                        <button
                           class="flex min-w-0 flex-1 items-center gap-1 text-left font-medium"
                           :aria-expanded="!collapsedConnections.has(connection.id)"
                           @click="toggleConnection(connection.id)"
                        >
                           <ChevronRight v-if="collapsedConnections.has(connection.id)" class="h-4 w-4 shrink-0" />
                           <ChevronDown v-else class="h-4 w-4 shrink-0" />
                           <Loader2 v-if="store.isConnecting(connection.id) || store.sessionOf(connection.id)?.refreshing" class="h-4 w-4 shrink-0 animate-spin" />
                           <Server v-else class="h-4 w-4 shrink-0 text-primary" />
                           <span class="truncate">{{ connection.name }}</span>
                        </button>

                        <template v-if="store.isConnecting(connection.id)">
                           <Tooltip>
                              <TooltipTrigger as-child>
                                 <Button variant="ghost" size="icon" class="h-6 w-6 hover:bg-destructive/10 hover:text-destructive" aria-label="Cancel connecting" @click="store.cancelConnect(connection.id)">
                                    <X class="h-3.5 w-3.5" />
                                 </Button>
                              </TooltipTrigger>
                              <TooltipContent side="bottom" class="text-xs">Cancel connecting</TooltipContent>
                           </Tooltip>
                        </template>
                        <div v-else class="flex gap-0.5 opacity-60 group-hover:opacity-100">
                           <Tooltip>
                              <TooltipTrigger as-child>
                                 <Button variant="ghost" size="icon" class="h-6 w-6" aria-label="Refresh" :disabled="store.sessionOf(connection.id)?.refreshing" @click="refresh(connection.id)">
                                    <RefreshCcw class="h-3.5 w-3.5" />
                                 </Button>
                              </TooltipTrigger>
                              <TooltipContent side="bottom" class="text-xs">Refresh</TooltipContent>
                           </Tooltip>
                           <Tooltip>
                              <TooltipTrigger as-child>
                                 <Button variant="ghost" size="icon" class="h-6 w-6" aria-label="Edit connection" @click="openEdit(connection)">
                                    <Pencil class="h-3.5 w-3.5" />
                                 </Button>
                              </TooltipTrigger>
                              <TooltipContent side="bottom" class="text-xs">Edit connection</TooltipContent>
                           </Tooltip>
                           <Tooltip>
                              <TooltipTrigger as-child>
                                 <Button variant="ghost" size="icon" class="h-6 w-6 hover:bg-red-600 hover:text-white" aria-label="Disconnect" @click="disconnect(connection.id)">
                                    <LogOut class="h-3.5 w-3.5" />
                                 </Button>
                              </TooltipTrigger>
                              <TooltipContent side="bottom" class="text-xs">Disconnect (closes its tabs)</TooltipContent>
                           </Tooltip>
                        </div>
                     </div>

                     <template v-if="!collapsedConnections.has(connection.id) && store.isConnected(connection.id)">
                        <div
                           v-if="store.sessionOf(connection.id)?.databases.length === 0"
                           class="py-1 ps-7 text-sm text-muted-foreground"
                        >
                           No databases found
                        </div>
                        <div v-for="database in store.sessionOf(connection.id)?.databases ?? []" :key="database" class="ps-3">
                           <div
                              class="flex cursor-pointer items-center gap-1 rounded-md px-2 py-1.5 text-sm transition-colors hover:bg-secondary"
                              @click="toggleDatabase(connection.id, database)"
                           >
                              <ChevronDown v-if="isDatabaseExpanded(connection.id, database)" class="h-4 w-4 shrink-0" />
                              <ChevronRight v-else class="h-4 w-4 shrink-0" />
                              <Database class="h-4 w-4 shrink-0" />
                              <span class="truncate">{{ database }}</span>
                           </div>
                           <template v-if="isDatabaseExpanded(connection.id, database)">
                              <div
                                 v-for="collection in collectionsOf(connection.id, database)"
                                 :key="collection"
                                 class="ms-5 flex cursor-pointer items-center gap-1 rounded-md px-2 py-1.5 text-sm transition-colors hover:bg-secondary"
                                 :class="{ 'bg-primary/15 text-primary font-medium': isCollectionActive(connection.id, database, collection) }"
                                 @click="handleCollectionSelect(connection.id, database, collection)"
                              >
                                 <FileJson class="h-4 w-4 shrink-0" />
                                 <span class="truncate">{{ collection }}</span>
                              </div>
                           </template>
                        </div>
                     </template>
                  </div>
               </TooltipProvider>
            </ScrollArea>
         </SidebarGroup>
      </SidebarContent>
   </Sidebar>

   <ConnectionDialog
      v-model:open="editOpen"
      mode="edit"
      :connection="editingConnection"
   />
</template>
