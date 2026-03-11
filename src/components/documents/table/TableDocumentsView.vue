<script setup lang="ts">
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { FlexRender } from '@tanstack/vue-table';
import { useVirtualizer } from '@tanstack/vue-virtual';
import { ArrowDown, ArrowUp, ArrowUpDown, GripVertical } from 'lucide-vue-next';
import { computed, ref, toRef } from 'vue';
import { JsonDocument } from '../models/types';
import { useDocumentTable } from '@/composables/table/useDocumentTable';
import { useColumnDragDrop } from '@/composables/table/useColumnDragDrop';

const ROW_HEIGHT = 33;
const TRUNCATE_MAX = 120;

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

const documentsRef = computed(() => props.documents);
const storageKeyRef = toRef(props, 'storageKey');

const { table, columnOrder, getFormatted } = useDocumentTable(documentsRef, storageKeyRef);
const {
   draggedColumnId,
   dropTargetColumnId,
   onDragStart,
   onDragOver,
   onDragLeave,
   onDrop,
   onDragEnd,
   onHeaderClick,
} = useColumnDragDrop(table, columnOrder);

const scrollContainerRef = ref<HTMLDivElement | null>(null);

const rows = computed(() => table.getRowModel().rows);

const virtualizer = useVirtualizer({
   get count() {
      return rows.value.length;
   },
   getScrollElement: () => scrollContainerRef.value,
   estimateSize: () => ROW_HEIGHT,
   overscan: 20,
});

function truncate(text: string): string {
   if (text.length <= TRUNCATE_MAX) return text;
   return text.slice(0, TRUNCATE_MAX) + '…';
}
</script>

<template>
   <div v-if="documents.length === 0" class="rounded-md border border-border p-3 text-sm text-muted-foreground">
      No documents found for this query.
   </div>

   <div v-else ref="scrollContainerRef" class="h-full overflow-auto rounded-md border border-border">
      <table class="grid text-sm" :style="{ minWidth: `${Math.max(table.getTotalSize(), 0)}px` }">
         <thead class="sticky top-0 z-10 grid bg-secondary">
            <tr class="flex w-full">
               <th
                  v-for="header in table.getFlatHeaders()"
                  :key="header.id"
                  :style="{ width: `${header.getSize()}px` }"
                  :class="[
                     'relative flex-none select-none whitespace-nowrap border-b border-r border-border px-3 py-2 text-left text-sm font-medium text-muted-foreground hover:text-foreground transition-colors',
                     draggedColumnId === header.column.id ? 'opacity-50' : '',
                     dropTargetColumnId === header.column.id ? 'bg-primary/10 border-l-2 border-l-primary' : '',
                  ]"
                  @click="onHeaderClick($event, header.column.getToggleSortingHandler())"
                  @dragover="onDragOver($event, header.column.id)"
                  @dragleave="onDragLeave"
                  @drop="onDrop($event, header.column.id)"
                  @dragend="onDragEnd"
               >
                  <div
                     class="flex items-center gap-1 pr-1"
                     draggable="true"
                     @dragstart="onDragStart($event, header.column.id)"
                  >
                     <GripVertical class="h-3 w-3 shrink-0 cursor-grab opacity-30 hover:opacity-70" />
                     <FlexRender :render="header.column.columnDef.header" :props="header.getContext()" />
                     <ArrowUp v-if="header.column.getIsSorted() === 'asc'" class="h-3 w-3" />
                     <ArrowDown v-else-if="header.column.getIsSorted() === 'desc'" class="h-3 w-3" />
                     <ArrowUpDown v-else class="h-3 w-3 opacity-30" />
                  </div>
                  <div
                     class="absolute right-0 top-0 h-full w-1 cursor-col-resize select-none touch-none transition-colors bg-border hover:bg-primary/50"
                     :class="header.column.getIsResizing() ? 'bg-primary' : ''"
                     draggable="false"
                     @mousedown.stop="header.getResizeHandler()($event)"
                     @touchstart.stop="header.getResizeHandler()($event)"
                     @click.stop
                     @dblclick.stop
                  />
               </th>
            </tr>
         </thead>
         <tbody class="relative grid" :style="{ height: `${virtualizer.getTotalSize()}px` }">
            <tr
               v-for="virtualRow in virtualizer.getVirtualItems()"
               :key="rows[virtualRow.index].id"
               :style="{ position: 'absolute', top: 0, left: 0, width: '100%', height: `${ROW_HEIGHT}px`, transform: `translateY(${virtualRow.start}px)` }"
               class="flex border-b border-border transition-colors hover:bg-muted/40"
            >
               <td
                  v-for="cell in rows[virtualRow.index].getVisibleCells()"
                  :key="cell.id"
                  :style="{ width: `${cell.column.getSize()}px`, height: `${ROW_HEIGHT}px` }"
                  class="flex-none border-r border-border px-3 py-1.5 font-mono text-sm text-foreground overflow-hidden"
               >
                  <template v-if="getFormatted(documents[rows[virtualRow.index].index], cell.column.id).length > TRUNCATE_MAX">
                     <TooltipProvider :delay-duration="300">
                        <Tooltip>
                           <TooltipTrigger as-child>
                              <span class="block truncate">{{ truncate(getFormatted(documents[rows[virtualRow.index].index], cell.column.id)) }}</span>
                           </TooltipTrigger>
                           <TooltipContent side="bottom" class="max-w-md whitespace-pre-wrap break-all text-xs">
                              {{ getFormatted(documents[rows[virtualRow.index].index], cell.column.id) }}
                           </TooltipContent>
                        </Tooltip>
                     </TooltipProvider>
                  </template>
                  <span v-else class="block truncate">
                     {{ getFormatted(documents[rows[virtualRow.index].index], cell.column.id) }}
                  </span>
               </td>
            </tr>
         </tbody>
      </table>
   </div>
</template>
