<script setup lang="ts">
import { ref, computed } from 'vue';
import { Check, X, Loader2, ChevronDown, ChevronRight } from 'lucide-vue-next';
import { Progress } from '@/components/ui/progress';
import type { TestStage } from '@/domains/connections';

const props = defineProps<{
  stages: TestStage[];
  error?: string | null;
  errorDetail?: string | null;
  isLoading?: boolean;
}>();

const showDetail = ref(false);

const progress = computed(() => {
  if (props.stages.length === 0) return 0;
  const completed = props.stages.filter((s) => s.status !== null).length;
  return Math.round((completed / props.stages.length) * 100);
});
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

    <Progress
      v-if="isLoading && stages.length > 0"
      :model-value="progress"
      class="mt-3"
    />

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
