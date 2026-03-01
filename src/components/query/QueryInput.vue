<template>
   <div class="query-input-wrapper">
      <div ref="query-editor"></div>
   </div>
</template>

<script setup lang="ts">
import { useMongoAutoComplete } from '@/domains/mongo';
import { autocompletion } from "@codemirror/autocomplete";
import { oneDark } from '@codemirror/theme-one-dark';
import { EditorView } from 'codemirror';
import { onMounted, ref, useTemplateRef, watch } from 'vue';

const props = defineProps<{ collectionProperties: string[] }>();
const query = defineModel<string>({ required: true });

const editorRef = useTemplateRef('query-editor')
const editor = ref<EditorView>();
const mongoAutoComplete = useMongoAutoComplete(props.collectionProperties);

onMounted(() => {
   editor.value = new EditorView({
      doc: query.value,
      parent: editorRef.value as Element,
      extensions: [
         oneDark,
         autocompletion({
            override: [mongoAutoComplete.autocomplete],
            activateOnTyping: true
         }),
         EditorView.updateListener.of(v => {
            query.value = v.state.doc.toString();
         })
      ]
   })
})

watch(() => props.collectionProperties, (newCollectionsProps) => {
   mongoAutoComplete.collectionProperties.value = newCollectionsProps;
})

</script>

<style scoped>
.query-input-wrapper {
  position: relative;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
}

/* Force CodeMirror into single-line mode */
.query-input-wrapper :deep(.cm-editor) {
  max-height: 38px;
}

.query-input-wrapper :deep(.cm-scroller) {
  overflow-x: auto;
  overflow-y: hidden;
  scrollbar-width: none;
}

.query-input-wrapper :deep(.cm-content) {
  padding: 8px 10px;
  white-space: pre;        /* prevents wrapping */
}

.query-error {
  display: block;
  padding: 4px 10px;
  font-size: 11px;
  color: #ff6b6b;
  background: #1a1a1a;
  border-top: 1px solid #3a3a3a;
}
</style>