<script setup lang="ts">
import { useTabsStore } from '@/stores/tabsStore';
import { cn } from '@/lib/utils';

const tabsStore = useTabsStore();

function setActiveTab(id: string) {
   tabsStore.switchTab(id);
}
</script>

<template>
   <div class="flex h-full flex-col">
      <div class="flex items-center border-b border-border bg-sidebar">
         <div class="flex flex-1 items-center overflow-x-auto">
            <template v-for="[id, tab] in tabsStore.tabs.entries()" :key="id">
               <div :class="cn(
                  'group flex cursor-pointer items-center gap-2 border-b-2 px-4 py-2 text-sm transition-colors',
                  tabsStore.getActiveTab()?.id === tab.id
                     ? 'border-primary bg-card text-foreground'
                     : 'border-transparent text-muted-foreground hover:text-foreground'
               )"
               @click="setActiveTab(id)"
               >
                  <span>{{ tab.context.collection }}</span>
               </div>
            </template>
         </div>
      </div>
   </div>
</template>