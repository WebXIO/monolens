<script setup lang="ts">
import { Button } from '@/components/ui/button';
import { json } from '@codemirror/lang-json';
import { oneDark } from '@codemirror/theme-one-dark';
import { basicSetup, EditorView } from 'codemirror';
import { ChevronDown, ChevronRight } from 'lucide-vue-next';
import { nextTick, onBeforeUnmount, ref, watch } from 'vue';
import { JsonDocument } from '../models/types';

const props = withDefaults(
   defineProps<{
      documents: JsonDocument[];
   }>(),
   {
      documents: () => [],
   },
);

const editors = new Map<number, EditorView>();
const containers = new Map<number, HTMLElement>();
const expandedIndexes = ref<Set<number>>(new Set());

function getDocumentText(index: number): string {
   return JSON.stringify(props.documents[index] ?? {}, null, 2);
}

function setEditorContainer(index: number, element: unknown) {
   if (element instanceof HTMLElement) {
      containers.set(index, element);
      if (isExpanded(index)) {
         createOrUpdateEditor(index, element);
      }
      return;
   }

   containers.delete(index);
   destroyEditor(index);
}

function createOrUpdateEditor(index: number, container: HTMLElement) {
   const content = getDocumentText(index);
   const existing = editors.get(index);

   if (existing) {
      if (existing.state.doc.toString() !== content) {
         existing.dispatch({
            changes: {
               from: 0,
               to: existing.state.doc.length,
               insert: content,
            },
         });
      }
      return;
   }

   const view = new EditorView({
      doc: content,
      extensions: [
         basicSetup,
         EditorView.lineWrapping,
         EditorView.editable.of(false),
         json(),
         oneDark,
      ],
      parent: container,
   });

   editors.set(index, view);
}

function destroyEditor(index: number) {
   const editor = editors.get(index);
   if (!editor) return;

   editor.destroy();
   editors.delete(index);
}

function destroyAllEditors() {
   for (const editor of editors.values()) {
      editor.destroy();
   }

   editors.clear();
}

function isExpanded(index: number): boolean {
   return expandedIndexes.value.has(index);
}

function toggleDocument(index: number) {
   const next = new Set(expandedIndexes.value);

   if (next.has(index)) {
      next.delete(index);
      expandedIndexes.value = next;
      destroyEditor(index);
      return;
   }

   next.add(index);
   expandedIndexes.value = next;

   nextTick(() => {
      const container = containers.get(index);
      if (!container) return;

      createOrUpdateEditor(index, container);
   });
}

function getDocumentTitle(document: JsonDocument, index: number): string {
   const id = document._id;
   if (id === undefined) {
      return `Document ${index + 1}`;
   }

   return `Document ${index + 1} · _id: ${JSON.stringify(id)}`;
}

watch(
   () => props.documents,
   (documents) => {
      if(!editors.keys()) return;
      expandedIndexes.value = new Set(documents.map((_, index) => index));

      for (const index of Array.from(editors.keys())) {
         if (index >= documents.length) {
            destroyEditor(index);
            containers.delete(index);
         }
      }

      nextTick(() => {
         for (const index of expandedIndexes.value) {
            const container = containers.get(index);
            if (!container) continue;

            createOrUpdateEditor(index, container);
         }
      });
   },
   { deep: true, immediate: true },
);

onBeforeUnmount(() => {
   destroyAllEditors();
   containers.clear();
});
</script>

<template>
   <div class="space-y-2">
      <div v-if="documents.length === 0" class="rounded-md border border-border p-3 text-sm text-muted-foreground">
         No documents found for this query.
      </div>

      <div v-for="(document, index) in documents" :key="index" class="overflow-hidden rounded-md border border-border">
         <div class="flex items-center justify-between border-b border-border px-2 py-1">
            <Button variant="ghost" size="sm" class="h-8 px-2 text-xs" @click="toggleDocument(index)">
               <ChevronDown v-if="isExpanded(index)" class="mr-1 h-4 w-4" />
               <ChevronRight v-else class="mr-1 h-4 w-4" />
               {{ getDocumentTitle(document, index) }}
            </Button>
         </div>

         <div v-show="isExpanded(index)" class="max-h-96 overflow-auto p-2">
            <div :ref="(element) => setEditorContainer(index, element)" class="rounded-md border border-border" />
         </div>
      </div>
   </div>
</template>