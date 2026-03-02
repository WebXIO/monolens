<template>
   <div class="query-input-wrapper">
      <div ref="query-editor"></div>
      <TooltipProvider v-if="queryIsNotEmpty">
         <Tooltip>
            <TooltipTrigger as-child>
               <button class="clear-button" @click="clearInput">
                  <X :size="14" />
               </button>
            </TooltipTrigger>
            <TooltipContent side="bottom" class="text-xs">
               Clear Query
            </TooltipContent>
         </Tooltip>
      </TooltipProvider>
   </div>
</template>

<script setup lang="ts">
import { useMongoAutoComplete } from '@/domains/mongo';
import { autocompletion } from "@codemirror/autocomplete";
import { oneDark } from '@codemirror/theme-one-dark';
import { EditorView } from 'codemirror';
import { X } from 'lucide-vue-next';
import { computed, onMounted, ref, useTemplateRef, watch } from 'vue';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';

const props = defineProps<{ collectionProperties: string[] }>();
const query = defineModel<string>({ required: true });

const editorRef = useTemplateRef('query-editor')
const editor = ref<EditorView>();
const mongoAutoComplete = useMongoAutoComplete(props.collectionProperties);
const emptyQuery = '{ }';

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

const queryIsNotEmpty = computed(() => {
   return query.value.trim() !== '' && query.value.trim() !== '{}' && query.value.trim() !== emptyQuery;
})

function clearInput() {
   query.value = emptyQuery;
   if (editor.value) {
      editor.value.dispatch({
         changes: { from: 0, to: editor.value.state.doc.length, insert: emptyQuery }
      });
   }
}

</script>

<style scoped>
.query-input-wrapper {
   position: relative;
   border: 1px solid #3a3a3a;
   border-radius: 6px;
}

.clear-button {
   position: absolute;
   right: 6px;
   top: 50%;
   transform: translateY(-50%);
   display: flex;
   align-items: center;
   justify-content: center;
   padding: 2px;
   border: none;
   background: transparent;
   color: #888;
   cursor: pointer;
   border-radius: 4px;
   transition: color 0.15s, background 0.15s;
}

.clear-button:hover {
   color: #ff6b6b;
   background: rgba(255, 107, 107, 0.1);
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
   white-space: pre;
   /* prevents wrapping */
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