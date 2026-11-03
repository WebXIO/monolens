<script setup lang="ts">
import { computed } from 'vue';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import Button from '@/components/ui/button/Button.vue';
import { useConfirm } from '@/composables/useConfirm';

const { state, answer } = useConfirm();

const open = computed({
  get: () => state.open,
  // closing with Escape, the X or a click outside means no
  set: (value: boolean) => {
    if (!value) answer(false);
  },
});
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent v-if="state.request" class="sm:max-w-[440px]">
      <DialogHeader>
        <DialogTitle>{{ state.request.title }}</DialogTitle>
        <DialogDescription v-if="state.request.description">{{ state.request.description }}</DialogDescription>
      </DialogHeader>
      <DialogFooter class="gap-2">
        <Button type="button" variant="outline" @click="answer(false)">Cancel</Button>
        <Button
          :variant="state.request.destructive ? 'destructive' : 'default'"
          type="button"
          @click="answer(true)"
        >
          {{ state.request.confirmLabel ?? 'Confirm' }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
