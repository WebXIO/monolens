import {
   createColumnHelper,
   getCoreRowModel,
   getSortedRowModel,
   useVueTable,
   type ColumnOrderState,
   type SortingState,
} from '@tanstack/vue-table';
import { useLocalStorage } from '@vueuse/core';
import { computed, ref, watch, type Ref } from 'vue';
import { type JsonDocument } from '@/components/documents/models/types';
import { useColumnSizing, MIN_COL_WIDTH, MAX_COL_WIDTH } from './useColumnSizing';
import { useFormattedValueCache } from './useFormattedValueCache';

function mergeColumnOrder(saved: string[], current: string[]): string[] {
   const currentSet = new Set(current);
   const ordered = saved.filter((key) => currentSet.has(key));
   const orderedSet = new Set(ordered);
   for (const key of current) {
      if (!orderedSet.has(key)) ordered.push(key);
   }
   return ordered;
}

export function useDocumentTable(documents: Ref<JsonDocument[]>, storageKey: Ref<string>) {
   const sorting = ref<SortingState>([]);
   const columnOrder = ref<ColumnOrderState>([]);

   const savedColumnOrder = useLocalStorage<string[]>(
      computed(() => storageKey.value ? `monolens:columnOrder:${storageKey.value}` : 'monolens:columnOrder:__empty__'),
      [],
   );

   const columnKeys = computed(() => {
      const keySet = new Set<string>();
      for (const doc of documents.value) {
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

   const { getFormatted } = useFormattedValueCache(documents);
   const { columnSizing, columnWidths, onColumnSizingChange } = useColumnSizing(documents, columnKeys, getFormatted);

   const columns = computed(() =>
      columnKeys.value.map((key) =>
         columnHelper.accessor((row) => row[key], {
            id: key,
            header: key,
            cell: (info) => getFormatted(documents.value[info.row.index], info.column.id),
            sortingFn: 'auto',
            size: columnWidths.value[key] ?? 150,
            minSize: MIN_COL_WIDTH,
            maxSize: MAX_COL_WIDTH,
         }),
      ),
   );

   const table = useVueTable({
      get data() {
         return documents.value;
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
         get columnSizing() {
            return columnSizing.value;
         },
      },
      enableColumnResizing: true,
      columnResizeMode: 'onChange',
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
      onColumnSizingChange,
      getCoreRowModel: getCoreRowModel(),
      getSortedRowModel: getSortedRowModel(),
   });

   return {
      table,
      columnOrder,
      getFormatted,
   };
}