export function formatCellValue(value: unknown): string {
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