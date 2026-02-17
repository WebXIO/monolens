<script setup lang="ts">
import { Check, X, Loader2 } from 'lucide-vue-next';
import type { TestStage } from '@/domains/connections';

defineProps<{
  stages: TestStage[];
  error?: string | null;
  isLoading?: boolean;
}>();
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
      <p class="text-sm text-destructive whitespace-pre-wrap">{{ error }}</p>
    </div>
  </div>
</template>
