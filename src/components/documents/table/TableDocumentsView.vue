<script setup lang="ts">
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import {
   createColumnHelper,
   FlexRender,
   getCoreRowModel,
   getSortedRowModel,
   useVueTable,
   type ColumnOrderState,
   type SortingState,
} from '@tanstack/vue-table';
import { ArrowDown, ArrowUp, ArrowUpDown, GripVertical } from 'lucide-vue-next';
import { useLocalStorage } from '@vueuse/core';
import { computed, ref, watch } from 'vue';
import { JsonDocument } from '../models/types';

const props = withDefaults(
   defineProps<{
      documents: JsonDocument[];
      storageKey?: string;
   }>(),
   {
      documents: () => [],
      storageKey: '',
   },
);

const sorting = ref<SortingState>([]);
const draggedColumnId = ref<string | null>(null);
const dropTargetColumnId = ref<string | null>(null);
const didDrag = ref(false);

// Persisted column order keyed by collection
const savedColumnOrder = useLocalStorage<string[]>(
   computed(() => props.storageKey ? `monolens:columnOrder:${props.storageKey}` : 'monolens:columnOrder:__empty__'),
   [],
);

const columnOrder = ref<ColumnOrderState>([]);

const columnKeys = computed(() => {
   const keySet = new Set<string>();
   for (const doc of props.documents) {
      for (const key of Object.keys(doc)) {
         keySet.add(key);
      }
   }
   const keys = [...keySet];
   keys.sort((a, b) => {
      if (a === '_id') return -1;
      if (b === '_id') return 1;
      return a.localeCompare(b);
   });
   return keys;
});

// Merge saved order with current keys: keep saved entries that still exist (in saved order),
// then append any new keys not in saved list. Removes stale keys automatically.
function mergeColumnOrder(saved: string[], current: string[]): string[] {
   const currentSet = new Set(current);
   const ordered = saved.filter((key) => currentSet.has(key));
   const orderedSet = new Set(ordered);
   for (const key of current) {
      if (!orderedSet.has(key)) ordered.push(key);
   }
   return ordered;
}

// Sync column order when document keys or saved order change
watch(
   [columnKeys, savedColumnOrder],
   ([keys, saved]) => {
      if (saved.length > 0) {
         columnOrder.value = mergeColumnOrder(saved, keys);
      } else {
         columnOrder.value = keys;
      }
   },
   { immediate: true },
);

const columnHelper = createColumnHelper<JsonDocument>();

const columns = computed(() =>
   columnKeys.value.map((key) =>
      columnHelper.accessor((row) => row[key], {
         id: key,
         header: key,
         cell: (info) => formatCellValue(info.getValue()),
         sortingFn: 'auto',
      }),
   ),
);

const table = useVueTable({
   get data() {
      return props.documents;
   },
   get columns() {
      return columns.value;
   },
   state: {
      get sorting() {
         return sorting.value;
      },
      get columnOrder() {
         return columnOrder.value;
      },
   },
   onSortingChange: (updaterOrValue) => {
      sorting.value =
         typeof updaterOrValue === 'function'
            ? updaterOrValue(sorting.value)
            : updaterOrValue;
   },
   onColumnOrderChange: (updaterOrValue) => {
      const newOrder =
         typeof updaterOrValue === 'function'
            ? updaterOrValue(columnOrder.value)
            : updaterOrValue;
      columnOrder.value = newOrder;
      savedColumnOrder.value = [...newOrder];
   },
   getCoreRowModel: getCoreRowModel(),
   getSortedRowModel: getSortedRowModel(),
});

// --- Drag-and-drop handlers ---
function onDragStart(event: DragEvent, columnId: string) {
   draggedColumnId.value = columnId;
   didDrag.value = false;
   if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move';
      event.dataTransfer.setData('text/plain', columnId);
   }
}

function onDragOver(event: DragEvent, columnId: string) {
   event.preventDefault();
   if (draggedColumnId.value && draggedColumnId.value !== columnId) {
      dropTargetColumnId.value = columnId;
   }
}

function onDragLeave() {
   dropTargetColumnId.value = null;
}

function onDrop(event: DragEvent, targetColumnId: string) {
   event.preventDefault();
   dropTargetColumnId.value = null;

   const sourceId = draggedColumnId.value;
   if (!sourceId || sourceId === targetColumnId) return;

   didDrag.value = true;
   const currentOrder = [...columnOrder.value];
   const sourceIndex = currentOrder.indexOf(sourceId);
   const targetIndex = currentOrder.indexOf(targetColumnId);
   if (sourceIndex === -1 || targetIndex === -1) return;

   currentOrder.splice(sourceIndex, 1);
   currentOrder.splice(targetIndex, 0, sourceId);

   table.setColumnOrder(currentOrder);
}

function onDragEnd() {
   draggedColumnId.value = null;
   dropTargetColumnId.value = null;
}

function onHeaderClick(event: MouseEvent, handler?: (e: MouseEvent) => void) {
   if (didDrag.value) {
      didDrag.value = false;
      return;
   }
   handler?.(event);
}

function formatCellValue(value: unknown): string {
   if (value === undefined) return '';
   if (value === null) return 'null';
   if (typeof value === 'object' && value !== null) {
      const obj = value as Record<string, unknown>;
      if ('$oid' in obj && typeof obj.$oid === 'string') return obj.$oid;   // to get the actual _id value
      // Extended JSON: Date
      if ('$date' in obj) {
         const d = obj.$date;
         if (typeof d === 'string') return d;
         if (typeof d === 'object' && d !== null && '$numberLong' in (d as Record<string, unknown>)) {
            return new Date(Number((d as Record<string, unknown>).$numberLong)).toISOString();
         }
      }
      // Extended JSON: NumberLong / NumberInt / NumberDecimal / NumberDouble
      if ('$numberLong' in obj && typeof obj.$numberLong === 'string') return obj.$numberLong;
      if ('$numberInt' in obj && typeof obj.$numberInt === 'string') return obj.$numberInt;
      if ('$numberDecimal' in obj && typeof obj.$numberDecimal === 'string') return obj.$numberDecimal;
      if ('$numberDouble' in obj && typeof obj.$numberDouble === 'string') return obj.$numberDouble;
      // Extended JSON: Timestamp
      if ('$timestamp' in obj) return JSON.stringify(obj.$timestamp);
      // Extended JSON: Binary
      if ('$binary' in obj) return '[Binary]';
      // Extended JSON: Regex
      if ('$regularExpression' in obj) {
         const re = obj.$regularExpression as Record<string, string>;
         return `/${re.pattern ?? ''}/${re.options ?? ''}`;
      }
      return JSON.stringify(value);
   }
   return String(value);
}

function truncate(text: string, max = 120): string {
   if (text.length <= max) return text;
   return text.slice(0, max) + '…';
}

function needsTruncation(text: string, max = 120): boolean {
   return text.length > max;
}
</script>

<template>
   <div v-if="documents.length === 0" class="rounded-md border border-border p-3 text-sm text-muted-foreground">
      No documents found for this query.
   </div>

   <div v-else class="overflow-auto rounded-md border border-border">
      <table class="w-full text-sm">
         <thead class="sticky top-0 z-10 bg-secondary">
            <tr>
               <th
                  v-for="header in table.getFlatHeaders()"
                  :key="header.id"
                  draggable="true"
                  :class="[
                     'select-none whitespace-nowrap border-b border-r border-border px-3 py-2 text-left text-xs font-medium text-muted-foreground hover:text-foreground transition-colors',
                     draggedColumnId === header.column.id ? 'opacity-50' : '',
                     dropTargetColumnId === header.column.id ? 'bg-primary/10 border-l-2 border-l-primary' : '',
                  ]"
                  @click="onHeaderClick($event, header.column.getToggleSortingHandler())"
                  @dragstart="onDragStart($event, header.column.id)"
                  @dragover="onDragOver($event, header.column.id)"
                  @dragleave="onDragLeave"
                  @drop="onDrop($event, header.column.id)"
                  @dragend="onDragEnd"
               >
                  <div class="flex items-center gap-1">
                     <GripVertical class="h-3 w-3 shrink-0 cursor-grab opacity-30 hover:opacity-70" />
                     <FlexRender :render="header.column.columnDef.header" :props="header.getContext()" />
                     <ArrowUp v-if="header.column.getIsSorted() === 'asc'" class="h-3 w-3" />
                     <ArrowDown v-else-if="header.column.getIsSorted() === 'desc'" class="h-3 w-3" />
                     <ArrowUpDown v-else class="h-3 w-3 opacity-30" />
                  </div>
               </th>
            </tr>
         </thead>
         <tbody>
            <tr
               v-for="row in table.getRowModel().rows"
               :key="row.id"
               class="border-b border-border transition-colors hover:bg-muted/40"
            >
               <td
                  v-for="cell in row.getVisibleCells()"
                  :key="cell.id"
                  class="max-w-xs border-r border-border px-3 py-1.5 font-mono text-xs text-foreground"
               >
                  <TooltipProvider v-if="needsTruncation(formatCellValue(cell.getValue()))" :delay-duration="300">
                     <Tooltip>
                        <TooltipTrigger as-child>
                           <span class="block truncate">{{ truncate(formatCellValue(cell.getValue())) }}</span>
                        </TooltipTrigger>
                        <TooltipContent side="bottom" class="max-w-md whitespace-pre-wrap break-all text-xs">
                           {{ formatCellValue(cell.getValue()) }}
                        </TooltipContent>
                     </Tooltip>
                  </TooltipProvider>
                  <span v-else class="block truncate">
                     <FlexRender :render="cell.column.columnDef.cell" :props="cell.getContext()" />
                  </span>
               </td>
            </tr>
         </tbody>
      </table>
   </div>
</template>
