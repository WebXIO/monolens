<script setup lang="ts">
import { computed } from 'vue';
import Logo from './Logo.vue';
import ConnectionSelector from '@/components/connection/ConnectionSelector.vue';
import { useConnectionStore } from '@/stores/connectionStore';

const store = useConnectionStore();

const activeConnection = computed(() => store.activeConnection.value);
</script>

<template>
   <header class="flex h-(--header-height) items-center justify-between border-b border-border bg-sidebar px-3">
      <div class="flex items-center gap-4">
         <Logo />

         <div class="h-5 w-px bg-border"></div>

         <ConnectionSelector v-if="activeConnection" />
      </div>
   </header>
</template>