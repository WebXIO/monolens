<script setup lang="ts">
import { Database } from 'lucide-vue-next';

</script>

<template>
   <div className="flex items-center gap-2">
      <div className="flex h-7 w-7 items-center justify-center rounded-md bg-primary">
         <Database className="h-4 w-4 text-primary-foreground" />
      </div>
      <span className="text-sm font-semibold">Monolens</span>
   </div>
</template>