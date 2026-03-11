import { type ColumnSizingState } from '@tanstack/vue-table';
import { computed, ref, type Ref } from 'vue';
import { type JsonDocument } from '@/components/documents/models/types';

export const MIN_COL_WIDTH = 60;
export const MAX_COL_WIDTH = 500;

const MONO_CHAR_WIDTH = 8.4;
const HEADER_CHAR_WIDTH = 8;
const CELL_PADDING = 24;
const HEADER_ICONS_WIDTH = 44;
const COLUMN_SIZING_SAMPLE_SIZE = 500;

function computeColumnWidths(
   documents: JsonDocument[],
   keys: string[],
   getFormatted: (doc: JsonDocument, key: string) => string,
): Record<string, number> {
   const widths: Record<string, number> = {};
   const sampleDocs = documents.length > COLUMN_SIZING_SAMPLE_SIZE
      ? documents.slice(0, COLUMN_SIZING_SAMPLE_SIZE)
      : documents;

   for (const key of keys) {
      const headerWidth = key.length * HEADER_CHAR_WIDTH + CELL_PADDING + HEADER_ICONS_WIDTH;

      const lengths: number[] = [];
      for (const doc of sampleDocs) {
         lengths.push(getFormatted(doc, key).length);
      }

      let dataWidth = 0;
      if (lengths.length > 0) {
         lengths.sort((a, b) => a - b);
         const p90Index = Math.floor(lengths.length * 0.9);
         const p90Length = lengths[Math.min(p90Index, lengths.length - 1)];
         dataWidth = p90Length * MONO_CHAR_WIDTH + CELL_PADDING;
      }

      widths[key] = Math.max(MIN_COL_WIDTH, Math.min(MAX_COL_WIDTH, Math.max(headerWidth, dataWidth)));
   }
   return widths;
}

export function useColumnSizing(
   documents: Ref<JsonDocument[]>,
   columnKeys: Ref<string[]>,
   getFormatted: (doc: JsonDocument, key: string) => string,
) {
   const columnSizing = ref<ColumnSizingState>({});

   const columnWidths = computed(() => computeColumnWidths(documents.value, columnKeys.value, getFormatted));

   function onColumnSizingChange(updaterOrValue: ColumnSizingState | ((old: ColumnSizingState) => ColumnSizingState)) {
      columnSizing.value =
         typeof updaterOrValue === 'function'
            ? updaterOrValue(columnSizing.value)
            : updaterOrValue;
   }

   return {
      columnSizing,
      columnWidths,
      onColumnSizingChange,
   };
}
