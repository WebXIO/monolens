<script setup lang="ts">
import { ref, computed } from 'vue';
import { Check, X, Loader2, Circle, ChevronDown, ChevronRight } from 'lucide-vue-next';
import {
  Stepper,
  StepperItem,
  StepperTrigger,
  StepperIndicator,
  StepperTitle,
  StepperSeparator,
} from '@/components/ui/stepper';
import type { TestStage } from '@/domains/connections';

const props = defineProps<{
  stages: TestStage[];
  error?: string | null;
  errorDetail?: string | null;
  isLoading?: boolean;
}>();

const showDetail = ref(false);

const currentStep = computed(() => {
  const failedIdx = props.stages.findIndex((s) => s.status === false);
  if (failedIdx !== -1) return failedIdx + 1;

  const pendingIdx = props.stages.findIndex((s) => s.status === null);
  if (pendingIdx !== -1) return pendingIdx + 1;

  return props.stages.length + 1;
});

function isActiveStage(index: number) {
  return props.isLoading && currentStep.value === index + 1;
}

function indicatorClass(stage: TestStage, index: number) {
  if (stage.status === true) {
    return 'bg-success text-success-foreground';
  }
  if (stage.status === false) {
    return 'bg-destructive text-destructive-foreground';
  }
  if (isActiveStage(index)) {
    return 'border-2 border-primary text-primary';
  }
  return 'border-2 border-muted-foreground/20 text-muted-foreground/30';
}

function separatorClass(stage: TestStage) {
  if (stage.status === true) return 'bg-success';
  if (stage.status === false) return 'bg-destructive';
  return 'bg-muted-foreground/20';
}

function titleClass(stage: TestStage) {
  if (stage.status === true) return 'text-foreground';
  if (stage.status === false) return 'text-destructive';
  return 'text-muted-foreground';
}

function itemAlignClass(index: number) {
  if (index === 0) return 'items-start';
  if (index === props.stages.length - 1) return 'items-end';
  return 'items-center';
}
</script>

<template>
  <div>
    <Stepper
      :model-value="currentStep"
      orientation="horizontal"
      class="!gap-0 w-full"
    >
      <template v-for="(stage, index) in stages" :key="index">
        <StepperItem
          :step="index + 1"
          :class="['!gap-0 flex-col flex-1', itemAlignClass(index)]"
        >
          <StepperTrigger
            as="div"
            class="!p-0 !rounded-none pointer-events-none flex-row items-center !gap-0 w-full"
          >
            <StepperSeparator
              v-if="index > 0"
              :class="separatorClass(stages[index - 1])"
              class="!h-0.5 flex-1 rounded-full"
            />
            <StepperIndicator :class="indicatorClass(stage, index)">
              <Check v-if="stage.status === true" class="size-3.5" />
              <X v-else-if="stage.status === false" class="size-3.5" />
              <Loader2
                v-else-if="isActiveStage(index)"
                class="size-3.5 animate-spin"
              />
              <Circle v-else class="size-2 fill-current" />
            </StepperIndicator>
            <StepperSeparator
              v-if="index < stages.length - 1"
              :class="separatorClass(stage)"
              class="!h-0.5 flex-1 rounded-full"
            />
            <div v-else class="flex-1" />
          </StepperTrigger>

          <StepperTitle
            :class="titleClass(stage)"
            class="!text-xs !font-normal mt-1.5 text-center whitespace-nowrap"
          >
            {{ stage.title }}
          </StepperTitle>
        </StepperItem>
      </template>
    </Stepper>

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
