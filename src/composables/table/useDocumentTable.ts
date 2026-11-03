import {
   columnOrderingFeature,
   columnResizingFeature,
   columnSizingFeature,
   columnVisibilityFeature,
   createColumnHelper,
   createSortedRowModel,
   rowSortingFeature,
   sortFn_alphanumeric,
   sortFn_basic,
   sortFn_datetime,
   sortFn_text,
   tableFeatures,
   useTable,
   type ColumnDef,
   type ColumnOrderState,
   type SortingState,
   type Updater,
} from '@tanstack/vue-table';
import { useLocalStorage } from '@vueuse/core';
import { computed, ref, watch, type Ref } from 'vue';
import { type JsonDocument } from '@/components/documents/models/types';
import { useColumnSizing, MIN_COL_WIDTH, MAX_COL_WIDTH } from './useColumnSizing';
import { useFormattedValueCache } from './useFormattedValueCache';
import { fieldNames } from '@/utils/documentFields';

// v9 only exposes the APIs of registered features; 'auto' sorting picks from the registered sortFns
export const documentTableFeatures = tableFeatures({
   rowSortingFeature,
   columnOrderingFeature,
   columnSizingFeature,
   columnResizingFeature,
   columnVisibilityFeature,
   sortedRowModel: createSortedRowModel(),
   sortFns: {
      alphanumeric: sortFn_alphanumeric,
      text: sortFn_text,
      datetime: sortFn_datetime,
      basic: sortFn_basic,
   },
});

export type DocumentTableFeatures = typeof documentTableFeatures;

export function mergeColumnOrder(saved: string[], current: string[]): string[] {
   const currentSet = new Set(current);
   const ordered = saved.filter((key) => currentSet.has(key));
   const orderedSet = new Set(ordered);
   for (const key of current) {
      if (!orderedSet.has(key)) ordered.push(key);
   }
   return ordered;
}

function resolve<T>(updater: Updater<T>, current: T): T {
   return typeof updater === 'function' ? (updater as (old: T) => T)(current) : updater;
}

export function useDocumentTable(documents: Ref<JsonDocument[]>, storageKey: Ref<string>) {
   const sorting = ref<SortingState>([]);
   const columnOrder = ref<ColumnOrderState>([]);

   const savedColumnOrder = useLocalStorage<string[]>(
      computed(() => storageKey.value ? `monolens:columnOrder:${storageKey.value}` : 'monolens:columnOrder:__empty__'),
      [],
   );

   const columnKeys = computed(() => fieldNames(documents.value));

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

   const columnHelper = createColumnHelper<DocumentTableFeatures, JsonDocument>();

   const { getFormatted } = useFormattedValueCache(documents);
   const { columnSizing, columnWidths, onColumnSizingChange } = useColumnSizing(documents, columnKeys, getFormatted);

   const columns = computed<readonly ColumnDef<DocumentTableFeatures, JsonDocument, unknown>[]>(() =>
      columnKeys.value.map((key) =>
         columnHelper.accessor((row) => row[key], {
            id: key,
            header: key,
            cell: (info) => getFormatted(documents.value[info.row.index], info.column.id),
            sortFn: 'auto',
            size: columnWidths.value[key] ?? 150,
            minSize: MIN_COL_WIDTH,
            maxSize: MAX_COL_WIDTH,
         }),
      ),
   );

   const table = useTable<DocumentTableFeatures, JsonDocument>({
      features: documentTableFeatures,
      data: documents,
      columns,
      state: computed(() => ({
         sorting: sorting.value,
         columnOrder: columnOrder.value,
         columnSizing: columnSizing.value,
      })),
      columnResizeMode: 'onChange',
      onSortingChange: (updater: Updater<SortingState>) => {
         sorting.value = resolve(updater, sorting.value);
      },
      onColumnOrderChange: (updater: Updater<ColumnOrderState>) => {
         const newOrder = resolve(updater, columnOrder.value);
         columnOrder.value = newOrder;
         savedColumnOrder.value = [...newOrder];
      },
      onColumnSizingChange,
   });

   return {
      table,
      columnOrder,
      getFormatted,
   };
}
