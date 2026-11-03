<script setup lang="ts">
import JSONDocumentsEditor from '@/components/documents/json/JSONDocumentsEditor.vue';
import { DocumentViewMode } from '@/components/documents/models/types';
import TableDocumentsView from '@/components/documents/table/TableDocumentsView.vue';
import QueryInput from '@/components/query/QueryInput.vue';
import { pageInfo, parseQueryTexts, type ParsedQuery } from '@/components/query/queryState';
import { fieldNames } from '@/utils/documentFields';
import { Button } from "@/components/ui/button";
import {
   DropdownMenu,
   DropdownMenuContent,
   DropdownMenuItem,
   DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Label } from '@/components/ui/label';
import ScrollArea from '@/components/ui/scroll-area/ScrollArea.vue';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { TaskCancelledError } from '@/composables/useAbortableCommand';
import { useDomain } from '@/domains';
import { ConnectionServiceIPC } from '@/domains/connections';
import { FindDocumentsResult } from '@/domains/connections/models/findDocumentsResult';
import { CommandRegistry, Commands } from '@/domains/shortcuts';
import { useTabsStore } from '@/stores/tabsStore';
import { extractErrorMessage } from '@/utils/errorMessage';
import { Braces, ChevronLeft, ChevronRight, Loader2, Play, RefreshCcw, TableIcon, X } from '@lucide/vue';
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { toast } from "vue-sonner";

const props = defineProps<{ tabId: string }>();

const commandHandler = useDomain<CommandRegistry>(CommandRegistry);
const connectionService = useDomain<ConnectionServiceIPC>(ConnectionServiceIPC);
const tabsStore = useTabsStore();

// the tab this view belongs to, not the active one
const tab = computed(() => tabsStore.tabs.get(props.tabId) ?? null);

const loading = ref(false);
const viewMode = ref<DocumentViewMode>(DocumentViewMode.TABLE);

const filterText = ref<string>('{ }');
const sortText = ref<string>('');
const projectionText = ref<string>('');
const limit = ref<number>(50);
const skip = ref<number>(0);
const result = ref<FindDocumentsResult>({ documents: [], totalCount: 0, totalCountEstimated: false });
// paging repeats the query that was run, not what is typed since
const lastQuery = ref<ParsedQuery | null>(null);
let abortController: AbortController | null = null;

const inputIds = {
   filter: `${props.tabId}-filter`,
   sort: `${props.tabId}-sort`,
   projection: `${props.tabId}-projection`,
};

const collectionProperties = ref<string[]>([]);

const limitOptions = [10, 25, 50, 100, 200];

const tableStorageKey = computed(() => {
   if (!tab.value) return '';
   const { connectionId, database, collection } = tab.value.context;
   return `${connectionId}:${database}:${collection}`;
});

const page = computed(() => pageInfo(
   skip.value,
   result.value.documents.length,
   result.value.totalCount,
   result.value.totalCountEstimated,
   limit.value,
));

async function run(query: ParsedQuery, pageSkip: number) {
   const context = tab.value?.context;
   if (!context) return;

   abortController?.abort();
   const controller = new AbortController();
   abortController = controller;
   loading.value = true;

   try {
      result.value = await connectionService.findDocuments(
         context.connectionId,
         {
            databaseName: context.database,
            collectionName: context.collection,
            filter: query.filter,
            sort: query.sort,
            projection: query.projection,
            skip: pageSkip,
            limit: limit.value,
         },
         { signal: controller.signal },
      );
      skip.value = pageSkip;
      lastQuery.value = query;
      if (result.value.documents.length) {
         collectionProperties.value = fieldNames(result.value.documents);
      }
   } catch (err) {
      // the old results stay; a cancel needs no message
      if (!(err instanceof TaskCancelledError)) {
         toast.error(extractErrorMessage(err));
      }
   } finally {
      if (abortController === controller) {
         abortController = null;
         loading.value = false;
      }
   }
}

/** What is typed, or null after showing why it can't be parsed. */
function parseInputs(): ParsedQuery | null {
   try {
      return parseQueryTexts({ filter: filterText.value, sort: sortText.value, projection: projectionText.value });
   } catch (err) {
      toast.error(err instanceof Error ? err.message : String(err));
      return null;
   }
}

/** Runs what is typed, from the first page. */
async function execute() {
   const query = parseInputs();
   if (query) await run(query, 0);
}

function cancel() {
   abortController?.abort();
}

// the limit only changes with a query that runs, so paging and label keep matching the shown result
function changeLimit(value: number) {
   const query = parseInputs();
   if (!query) return;
   limit.value = value;
   run(query, 0);
}

function previousPage() {
   if (!lastQuery.value) return;
   run(lastQuery.value, Math.max(0, skip.value - limit.value));
}

function nextPage() {
   if (!lastQuery.value) return;
   run(lastQuery.value, skip.value + limit.value);
}

// hidden tabs stay mounted, so mod+enter must only run the active one
function executeIfActive() {
   if (tabsStore.activeTab === props.tabId) execute();
}

onMounted(() => {
   commandHandler.subscribe(Commands.EVENT_EXECUTE, executeIfActive);
   execute();
})

onUnmounted(() => {
   commandHandler.unsubscribe(Commands.EVENT_EXECUTE, executeIfActive);
   abortController?.abort();
})

</script>

<template>
   <div class="flex h-full flex-col p-2">
      <TooltipProvider :delay-duration="300">
         <div class="grid grid-cols-12 items-end gap-2">
            <div class="col-span-9">
               <Label :for="inputIds.filter">Filter</Label>
               <QueryInput :id="inputIds.filter" v-model="filterText" :collection-properties="collectionProperties" />
            </div>
            <div class="col-span-3 flex items-center gap-1">
               <Tooltip v-if="loading">
                  <TooltipTrigger as-child>
                     <Button variant="outline" size="icon" class="h-9 w-9 hover:bg-destructive/10 hover:text-destructive" aria-label="Cancel query" @click="cancel">
                        <X class="h-4 w-4" />
                     </Button>
                  </TooltipTrigger>
                  <TooltipContent side="bottom" class="text-xs">Cancel</TooltipContent>
               </Tooltip>
               <Tooltip v-else>
                  <TooltipTrigger as-child>
                     <Button variant="default" size="icon" class="h-9 w-9" aria-label="Execute query" @click="execute">
                        <Play class="h-4 w-4" />
                     </Button>
                  </TooltipTrigger>
                  <TooltipContent side="bottom" class="text-xs">Execute (Ctrl/Cmd+Enter)</TooltipContent>
               </Tooltip>
               <DropdownMenu>
                  <DropdownMenuTrigger as-child>
                     <Button variant="outline" size="sm" class="h-9 text-xs tabular-nums">
                        Limit: {{ limit }}
                     </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                     <DropdownMenuItem v-for="opt in limitOptions" :key="opt" @click="changeLimit(opt)">
                        {{ opt }}
                     </DropdownMenuItem>
                  </DropdownMenuContent>
               </DropdownMenu>
               <Tooltip>
                  <TooltipTrigger as-child>
                     <Button variant="ghost" size="icon" class="h-9 w-9" aria-label="Refresh" :disabled="loading || !lastQuery" @click="lastQuery && run(lastQuery, skip)">
                        <Loader2 v-if="loading" class="h-4 w-4 animate-spin" />
                        <RefreshCcw v-else class="h-4 w-4" />
                     </Button>
                  </TooltipTrigger>
                  <TooltipContent side="bottom" class="text-xs">Refresh this page</TooltipContent>
               </Tooltip>

               <div class="ml-auto flex items-center rounded-md border border-border">
                  <Tooltip>
                     <TooltipTrigger as-child>
                        <Button
                           :variant="viewMode === DocumentViewMode.TABLE ? 'secondary' : 'ghost'"
                           size="icon"
                           class="hover:bg-info/10 hover:text-info hover:border-info/50 transition-colors"
                           aria-label="Table view"
                           @click="viewMode = DocumentViewMode.TABLE"
                        >
                           <TableIcon class="h-4 w-4" />
                        </Button>
                     </TooltipTrigger>
                     <TooltipContent side="bottom" class="text-xs">Table View</TooltipContent>
                  </Tooltip>
                  <Tooltip>
                     <TooltipTrigger as-child>
                        <Button
                           :variant="viewMode === DocumentViewMode.JSON ? 'secondary' : 'ghost'"
                           size="icon"
                           class="hover:bg-info/10 hover:text-info hover:border-info/50 transition-colors"
                           aria-label="JSON view"
                           @click="viewMode = DocumentViewMode.JSON"
                        >
                           <Braces class="h-4 w-4" />
                        </Button>
                     </TooltipTrigger>
                     <TooltipContent side="bottom" class="text-xs">JSON View</TooltipContent>
                  </Tooltip>
               </div>
            </div>
         </div>

         <div class="mt-1 grid grid-cols-12 items-end gap-2">
            <div class="col-span-5">
               <Label :for="inputIds.sort">Sort</Label>
               <QueryInput :id="inputIds.sort" v-model="sortText" :collection-properties="collectionProperties" placeholder="{ field: 1 }" />
            </div>
            <div class="col-span-4">
               <Label :for="inputIds.projection">Projection</Label>
               <QueryInput :id="inputIds.projection" v-model="projectionText" :collection-properties="collectionProperties" placeholder="{ field: 1 }" />
            </div>
            <div class="col-span-3 flex h-9 items-center justify-end gap-1 text-xs tabular-nums text-muted-foreground">
               <span :title="result.totalCountEstimated ? 'Estimated from the collection\'s metadata' : undefined">{{ page.label }}</span>
               <Button variant="ghost" size="icon" class="h-7 w-7" aria-label="Previous page" :disabled="loading || !page.hasPrevious" @click="previousPage">
                  <ChevronLeft class="h-4 w-4" />
               </Button>
               <Button variant="ghost" size="icon" class="h-7 w-7" aria-label="Next page" :disabled="loading || !page.hasNext" @click="nextPage">
                  <ChevronRight class="h-4 w-4" />
               </Button>
            </div>
         </div>
      </TooltipProvider>

      <div class="mt-2 shrink-0 border border-b border-primary"></div>
      <div class="flex-1 min-h-0 overflow-hidden">
         <ScrollArea v-if="viewMode === DocumentViewMode.JSON" class="h-full">
            <JSONDocumentsEditor :documents="result.documents" />
         </ScrollArea>
         <TableDocumentsView v-else :documents="result.documents" :storage-key="tableStorageKey" />
      </div>
   </div>
</template>
