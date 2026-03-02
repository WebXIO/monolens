<script setup lang="ts">
import Logo from './Logo.vue';
import ConnectionSelector from '@/components/connection/ConnectionSelector.vue';
import { useConnectionStore } from '@/stores/connectionStore';

const store = useConnectionStore();
</script>

<template>
   <header class="flex h-(--header-height) shrink-0 items-center justify-between border-b border-border bg-sidebar px-3">
      <div class="flex items-center gap-4">
         <Logo />

         <div class="h-5 w-px bg-border"></div>

         <ConnectionSelector v-if="store.activeConnection" />
      </div>
   </header>
</template>