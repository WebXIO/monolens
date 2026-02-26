<script setup lang="ts">
import JSONDocumentsEditor from '@/components/documents/json/JSONDocumentsEditor.vue';
import { Button } from "@/components/ui/button";
import {
   DropdownMenu,
   DropdownMenuContent,
   DropdownMenuItem,
   DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { useDomain } from '@/domains';
import { FindDocumentsResult } from '@/domains/connections/models/findDocumentsResult';
import { CommandRegistry, Commands } from '@/domains/shortcuts';
import { useConnectionStore } from '@/stores/connectionStore';
import { useTabsStore } from '@/stores/tabsStore';
import { parseMongoDbQuery } from '@/utils/queryParser';
import { invoke } from '@tauri-apps/api/core';
import { Loader2, Play, RefreshCcw } from 'lucide-vue-next';
import { onMounted, onUnmounted, ref } from 'vue';
import { toast } from "vue-sonner";

const commandHandler = useDomain<CommandRegistry>(CommandRegistry);
const tabsStore = useTabsStore();
const connectionStore = useConnectionStore();

const loading = ref(false);

const query = ref<string>('{ }');
const limit = ref<number>(50);
const result = ref<FindDocumentsResult>({ documents: [], totalCount: 0 });
const queryInputId = 'query-filter-input';

const limitOptions = [10, 25, 50, 100, 200];

async function execute() {
   if (!connectionStore.activeConnection) return;
   const tab = tabsStore.getActiveTab();
   if (!tab) return;

   if(!query.value) query.value = '{}'

   let queryParsed = {};

   try {
      console.log(query.value);
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
   } finally {
      loading.value = false;
   }
}

onMounted(() => {
   commandHandler.subscribe(Commands.EVENT_EXECUTE, execute);
})

onUnmounted(() => {
   commandHandler.unsubscribe(Commands.EVENT_EXECUTE, execute);
})

</script>

<template>
   <div class="p-2">
      <div class="grid grid-cols-12 items-end gap-2">
         <div class="col-span-9">
            <Label :for="queryInputId">Query</Label>
            <Input :id="queryInputId" v-model="query" />
         </div>
         <div class="col-span-3 flex items-center gap-1">
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
         </div>
      </div>

      <div class="mt-2 border border-b border-primary"></div>

      <div class="mt-3">
         <JSONDocumentsEditor :documents="result.documents" />
      </div>

   </div>
</template>
