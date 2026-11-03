<script setup lang="ts">
import { computed } from 'vue';
import { Check, X, Loader2, Circle, Minus } from '@lucide/vue';
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
  isLoading?: boolean;
}>();

const currentStep = computed(() => {
  const failedIdx = props.stages.findIndex((s) => s.status === 'failed');
  if (failedIdx !== -1) return failedIdx + 1;

  const pendingIdx = props.stages.findIndex((s) => s.status === 'pending');
  if (pendingIdx !== -1) return pendingIdx + 1;

  return props.stages.length + 1;
});

function isActiveStage(index: number) {
  return props.isLoading && currentStep.value === index + 1;
}

function indicatorClass(stage: TestStage, index: number) {
  if (stage.status === 'passed') {
    return 'bg-success text-success-foreground';
  }
  if (stage.status === 'failed') {
    return 'bg-destructive text-destructive-foreground';
  }
  if (stage.status === 'skipped') {
    return 'bg-warning/20 text-warning border-2 border-warning/50';
  }
  if (isActiveStage(index)) {
    return 'border-2 border-primary text-primary';
  }
  return 'border-2 border-muted-foreground/20 text-muted-foreground/30';
}

function separatorClass(stage: TestStage) {
  if (stage.status === 'passed') return 'bg-success';
  if (stage.status === 'skipped') return 'bg-warning/50';
  if (stage.status === 'failed') return 'bg-destructive';
  return 'bg-muted-foreground/20';
}

function titleClass(stage: TestStage) {
  if (stage.status === 'passed') return 'text-foreground';
  if (stage.status === 'failed') return 'text-destructive';
  if (stage.status === 'skipped') return 'text-warning';
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
    <div v-if="stages.length === 0 && isLoading" class="flex items-center gap-2 text-xs text-muted-foreground">
      <Loader2 class="size-3.5 animate-spin" />
      Starting the test…
    </div>
    <Stepper
      v-else
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
              <Check v-if="stage.status === 'passed'" class="size-3.5" />
              <X v-else-if="stage.status === 'failed'" class="size-3.5" />
              <Minus v-else-if="stage.status === 'skipped'" class="size-3.5" />
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
          <span v-if="stage.detail" class="mt-0.5 text-center text-[11px] leading-tight text-muted-foreground">
            {{ stage.detail }}
          </span>
        </StepperItem>
      </template>
    </Stepper>
  </div>
</template>
