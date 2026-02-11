<script setup lang="ts">
import { computed, ref } from 'vue';
import { SidebarProps, SidebarContent, SidebarGroup, Sidebar } from './ui/sidebar';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { Button } from "@/components/ui/button"
import { ChevronDown, Database, RefreshCcw, Loader2, ChevronUp, FileJson } from 'lucide-vue-next';
import { ScrollArea } from '@/components/ui/scroll-area';
import { useConnectionStore } from '@/stores/connectionStore';
import { invoke } from '@tauri-apps/api/core';
import { tryCatch } from '@/utils/result';

const props = defineProps<SidebarProps>()

const store = useConnectionStore();

const databases = computed(() => store.databases.value);
const isDatabasesLoading = computed(() => store.isDatabasesLoading.value);
const activeConnection = computed(() => store.activeConnection.value);

const toggledDatabases = ref<Set<string>>(new Set());
const collections = ref<Record<string, string[]>>({});

async function toggleDatabase(dbName: string) {

   if (!activeConnection.value) return;


   if (toggledDatabases.value.has(dbName)) {
      toggledDatabases.value.delete(dbName);
      return;
   }

   if (collections.value[dbName]) {
      toggledDatabases.value.add(dbName);
      return;
   }

   const { data, error } = await tryCatch<string[]>(invoke('list_collections', { connection: activeConnection.value, databaseName: dbName }))

   if (data) {
      collections.value[dbName] = data;
      toggledDatabases.value.add(dbName);
   }
   else
      console.error(error)

}

function isDatabaseExpended(name: string) {
   return toggledDatabases.value.has(name);
}

async function refreshDatabases() {
   if (activeConnection.value) {
      await store.connectTo(activeConnection.value);
   }
}
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
                        <Button variant="ghost" size="icon" class="h-6 w-6" :disabled="isDatabasesLoading"
                           @click="refreshDatabases">
                           <Loader2 v-if="isDatabasesLoading" class="h-3.5 w-3.5 animate-spin" />
                           <RefreshCcw v-else class="h-3.5 w-3.5" />
                        </Button>
                     </TooltipTrigger>
                     <TooltipContent side="bottom" class="text-xs">
                        Refresh
                     </TooltipContent>
                  </Tooltip>
               </TooltipProvider>
            </div>
         </SidebarGroup>
         <SidebarGroup>
            <ScrollArea class="flex-1">
               <div v-if="isDatabasesLoading" class="flex items-center gap-2 px-2 py-4 text-sm text-muted-foreground">
                  <Loader2 class="h-4 w-4 animate-spin" />
                  <span>Loading databases...</span>
               </div>

               <div v-else-if="databases.length === 0" class="px-2 py-4 text-sm text-muted-foreground">
                  No databases found
               </div>

               <div v-else v-for="database in databases" :key="database">
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
                        class="group flex cursor-pointer items-center gap-1 rounded-md px-2 py-1.5 text-sm transition-colors hover:bg-secondary">
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