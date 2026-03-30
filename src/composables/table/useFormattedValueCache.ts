import { watch, type Ref } from 'vue';
import { type JsonDocument } from '@/components/documents/models/types';
import { formatCellValue } from '@/utils/formatCellValues';

export function useFormattedValueCache(documents: Ref<JsonDocument[]>) {
   let cache = new WeakMap<JsonDocument, Map<string, string>>();

   watch(documents, () => {
      cache = new WeakMap();
   });

   function getFormatted(doc: JsonDocument, key: string): string {
      let docCache = cache.get(doc);
      if (!docCache) {
         docCache = new Map();
         cache.set(doc, docCache);
      }
      let value = docCache.get(key);
      if (value === undefined) {
         value = formatCellValue(doc[key]);
         docCache.set(key, value);
      }
      return value;
   }

   return { getFormatted };
}
