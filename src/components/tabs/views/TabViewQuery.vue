<script setup lang="ts">
import JSONDocumentsEditor from '@/components/documents/json/JSONDocumentsEditor.vue';
import { DocumentViewMode } from '@/components/documents/models/types';
import TableDocumentsView from '@/components/documents/table/TableDocumentsView.vue';
import QueryInput from '@/components/query/QueryInput.vue';
import { Button } from "@/components/ui/button";
import {
   DropdownMenu,
   DropdownMenuContent,
   DropdownMenuItem,
   DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Label } from '@/components/ui/label';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { useDomain } from '@/domains';
import { FindDocumentsResult } from '@/domains/connections/models/findDocumentsResult';
import { CommandRegistry, Commands } from '@/domains/shortcuts';
import { useConnectionStore } from '@/stores/connectionStore';
import { useTabsStore } from '@/stores/tabsStore';
import { parseMongoDbQuery } from '@/utils/queryParser';
import { invoke } from '@tauri-apps/api/core';
import { Braces, Loader2, Play, RefreshCcw, TableIcon } from 'lucide-vue-next';
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { toast } from "vue-sonner";

const commandHandler = useDomain<CommandRegistry>(CommandRegistry);
const tabsStore = useTabsStore();
const connectionStore = useConnectionStore();

const loading = ref(false);
const viewMode = ref<DocumentViewMode>(DocumentViewMode.TABLE);

const query = ref<string>('{ }');
const limit = ref<number>(50);
const result = ref<FindDocumentsResult>({ documents: [], totalCount: 0 });
const queryInputId = 'query-filter-input';

const collectionProperties = ref<string[]>([]);

const limitOptions = [10, 25, 50, 100, 200];

const tableStorageKey = computed(() => {
   const tab = tabsStore.getActiveTab();
   if (!tab || !connectionStore.activeConnection) return '';
   return `${connectionStore.activeConnection.id}:${tab.context.database}:${tab.context.collection}`;
});

async function execute() {
   if (!connectionStore.activeConnection) return;
   const tab = tabsStore.getActiveTab();
   if (!tab) return;

   if (!query.value) query.value = '{}'

   let queryParsed = {};

   try {
      queryParsed = parseMongoDbQuery(query.value)
   } catch (err) {
      toast.error(String(err))
      return;
   }

   loading.value = true;

   try {
      const id = await invoke('start_find_documents', {
         connection: connectionStore.activeConnection,
         databaseName: tab.context.database,
         collectionName: tab.context.collection,
         filter: queryParsed,
         skip: 0,
         limit: limit.value,
      });

      result.value = await invoke<FindDocumentsResult>('await_task_result', { id });

      if (result.value.documents.length) {
         collectionProperties.value = [...new Set(...result.value.documents.map((d) => Object.keys(d))).values()]
      }


   } finally {
      loading.value = false;
   }
}

onMounted(() => {
   commandHandler.subscribe(Commands.EVENT_EXECUTE, execute);
   execute();
})

onUnmounted(() => {
   commandHandler.unsubscribe(Commands.EVENT_EXECUTE, execute);
})

</script>

<template>
   <div class="flex h-full flex-col p-2">
      <div class="grid grid-cols-12 items-end gap-2">
         <div class="col-span-9">
            <Label :for="queryInputId">Query</Label>
            <QueryInput :id="queryInputId" v-model="query" :collection-properties="collectionProperties" />
         </div>
         <div class="col-span-3 flex items-center gap-1">
            <TooltipProvider>
               <Tooltip>
                  <TooltipTrigger as-child>
                     <Button variant="default" size="icon" class="h-9 w-9" :disabled="loading" @click="execute">
                        <Loader2 v-if="loading" class="h-4 w-4 animate-spin" />
                        <Play v-else class="h-4 w-4" />
                     </Button>
                  </TooltipTrigger>
                  <TooltipContent side="bottom" class="text-xs">
                     Execute Query
                  </TooltipContent>
               </Tooltip>
            </TooltipProvider>
            <DropdownMenu>
               <DropdownMenuTrigger as-child>
                  <Button variant="outline" size="sm" class="h-9 text-xs tabular-nums">
                     Limit: {{ limit }}
                  </Button>
               </DropdownMenuTrigger>
               <DropdownMenuContent align="end">
                  <DropdownMenuItem v-for="opt in limitOptions" :key="opt" @click="limit = opt">
                     {{ opt }}
                  </DropdownMenuItem>
               </DropdownMenuContent>
            </DropdownMenu>
            <TooltipProvider>
               <Tooltip>
                  <TooltipTrigger as-child>
                     <Button variant="ghost" size="icon" class="h-9 w-9" :disabled="loading" @click="execute">
                        <Loader2 v-if="loading" class="h-4 w-4 animate-spin" />
                        <RefreshCcw v-else class="h-4 w-4" />
                     </Button>
                  </TooltipTrigger>
                  <TooltipContent side="bottom" class="text-xs">
                     Refresh
                  </TooltipContent>
               </Tooltip>
            </TooltipProvider>

            <div class="ml-auto flex items-center rounded-md border border-border">
               <TooltipProvider>
                  <Tooltip>
                     <TooltipTrigger as-child>
                        <Button
                           :variant="viewMode === DocumentViewMode.TABLE ? 'secondary' : 'ghost'"
                           size="icon"
                           class="hover:bg-info/10 hover:text-info hover:border-info/50 transition-colors"
                           @click="viewMode = DocumentViewMode.TABLE"
                        >
                           <TableIcon class="h-4 w-4" />
                        </Button>
                     </TooltipTrigger>
                     <TooltipContent side="bottom" class="text-xs">
                        Table View
                     </TooltipContent>
                  </Tooltip>
               </TooltipProvider>
               <TooltipProvider>
                  <Tooltip>
                     <TooltipTrigger as-child>
                        <Button
                           :variant="viewMode === DocumentViewMode.JSON ? 'secondary' : 'ghost'"
                           size="icon"
                           class="hover:bg-info/10 hover:text-info hover:border-info/50 transition-colors"
                           @click="viewMode = DocumentViewMode.JSON"
                        >
                           <Braces class="h-4 w-4" />
                        </Button>
                     </TooltipTrigger>
                     <TooltipContent side="bottom" class="text-xs">
                        JSON View
                     </TooltipContent>
                  </Tooltip>
               </TooltipProvider>
            </div>
         </div>
      </div>

      <div class="mt-2 shrink-0 border border-b border-primary"></div>
      <ScrollArea class="flex-1 min-h-0">

         <JSONDocumentsEditor v-if="viewMode === DocumentViewMode.JSON" :documents="result.documents" />
         <TableDocumentsView v-else :documents="result.documents" :storage-key="tableStorageKey" />

      </ScrollArea>

   </div>
</template>
