<script setup lang="ts">
import { ref } from 'vue';
import { Check, X, Loader2, ChevronDown, ChevronRight } from 'lucide-vue-next';
import type { TestStage } from '@/domains/connections';

defineProps<{
  stages: TestStage[];
  error?: string | null;
  errorDetail?: string | null;
  isLoading?: boolean;
}>();

const showDetail = ref(false);
</script>

<template>
  <div class="space-y-2">
    <div v-for="(stage, index) in stages" :key="index" class="flex items-center gap-3 text-sm">
      <div class="w-5 h-5 flex items-center justify-center">
        <Check v-if="stage.status === true" class="w-4 h-4 text-green-500" />
        <X v-else-if="stage.status === false" class="w-4 h-4 text-red-500" />
        <Loader2 v-else-if="isLoading && stages.findIndex(s => s.status === null) === index"
          class="w-4 h-4 text-muted-foreground animate-spin" />
        <div v-else class="w-2 h-2 rounded-full bg-muted-foreground/30" />
      </div>

      <span :class="[
        stage.status === true ? 'text-foreground' : 'text-muted-foreground',
        stage.status === false ? 'text-red-500' : ''
      ]">
        {{ stage.title }}
      </span>
    </div>

    <div v-if="error" class="mt-3 p-3 bg-destructive/10 border border-destructive/20 rounded-md w-100 break-words">
      <p class="text-sm text-destructive">{{ error }}</p>
      <button
        v-if="errorDetail && errorDetail !== error"
        class="mt-2 flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors"
        @click="showDetail = !showDetail"
      >
        <ChevronDown v-if="showDetail" class="w-3 h-3" />
        <ChevronRight v-else class="w-3 h-3" />
        {{ showDetail ? 'Hide details' : 'Show details' }}
      </button>
      <pre
        v-if="showDetail && errorDetail"
        class="mt-2 text-xs text-muted-foreground whitespace-pre-wrap font-mono bg-muted/50 rounded p-2"
      >{{ errorDetail }}</pre>
    </div>
  </div>
</template>
