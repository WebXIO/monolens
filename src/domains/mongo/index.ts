import { CompletionContext } from "@codemirror/autocomplete";
import { ref } from "vue";

export const MONGO_OPERATORS = [
  { label: "$eq", detail: "Equals" },
  { label: "$ne", detail: "Not equals" },
  { label: "$gt", detail: "Greater than" },
  { label: "$gte", detail: "Greater than or equal" },
  { label: "$lt", detail: "Less than" },
  { label: "$lte", detail: "Less than or equal" },
  { label: "$in", detail: "In array" },
  { label: "$nin", detail: "Not in array" },
  { label: "$and", detail: "Logical AND" },
  { label: "$or", detail: "Logical OR" },
  { label: "$nor", detail: "Logical NOR" },
  { label: "$not", detail: "Logical NOT" },
  { label: "$exists", detail: "Field exists" },
  { label: "$type", detail: "BSON type" },
  { label: "$regex", detail: "Regular expression" },
  { label: "$elemMatch", detail: "Array element match" },
  { label: "$size", detail: "Array size" },
  { label: "$all", detail: "Array contains all" },
  { label: "$text", detail: "Text search" },
];

export const SHELL_FUNCTIONS = [
  { label: "ObjectId", detail: 'ObjectId("...")', apply: '("")' },
  { label: "ISODate", detail: 'ISODate("2024-01-01")', apply: '("")' },
  { label: "NumberLong", detail: "NumberLong(0)", apply: "()" },
  { label: "NumberDecimal", detail: "NumberDecimal(0.0)", apply: "()" },
  { label: "Timestamp", detail: "Timestamp(t, i)", apply: "()" },
  { label: "RegExp", detail: 'RegExp("pattern", "flags")', apply: '("")' },
];

export function useMongoAutoComplete(collectionProperties: string[]) {
  const _collectionProperties = ref<string[]>(collectionProperties);

  function autocomplete(context: CompletionContext) {
    const word = context.matchBefore(/[\w$]+/);

    if (!word && !context.explicit) return null;

    const before = context.state.doc.sliceString(0, context.pos);

    // After a colon → suggest operators + shell functions
    if (/:\s*[\w$]*$/.test(before)) {
      return {
        from: word ? word.from : context.pos,
        options: [
          ...MONGO_OPERATORS.map((op) => ({
            label: op.label,
            detail: op.detail,
            type: "keyword",
          })),
          ...SHELL_FUNCTIONS.map((fn) => ({
            label: fn.label,
            detail: fn.detail,
            type: "function",
            apply: fn.label + fn.apply, // auto-open parens + quote
          })),
        ],
      };
    }

    // After { or , → suggest collection field names + top-level operators
    if (/[{,]\s*[\w$]*$/.test(before)) {
      const fieldOptions = _collectionProperties.value.map((field) => ({
        label: field,
        detail: "field",
        type: "property",
      }));

      const topLevelOperators = ["$and", "$or", "$nor", "$text"].map((op) => ({
        label: op,
        detail: "top-level operator",
        type: "keyword",
      }));

      return {
        from: word ? word.from : context.pos,
        options: [
          ...fieldOptions,
          ...topLevelOperators,
          ...MONGO_OPERATORS.map((op) => ({
            label: op.label,
            detail: op.detail,
            type: "keyword",
          })),
        ],
      };
    }

    return null;
  }

  return {autocomplete, collectionProperties: _collectionProperties};
}
