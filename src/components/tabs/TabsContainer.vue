<script setup lang="ts">
import { useTabsStore } from '@/stores/tabsStore';
import { cn } from '@/lib/utils';
import { CircleX } from 'lucide-vue-next';
import { TabKind } from '@/domains/tabs';
import TabViewQuery from './views/TabViewQuery.vue';
import { computed } from 'vue';

const tabsStore = useTabsStore();

const currentTabType = computed(() => {
   return tabsStore.getActiveTab()?.type ?? null;
})

function setActiveTab(id: string) {
   tabsStore.switchTab(id);
}

function deleteTab(id: string, event?: MouseEvent) {
   tabsStore.deleteTab(id);
   event?.stopPropagation(); // prevent propagation so that the switch event is not called after deletion
}

const components = {
   [TabKind.QUERY]: TabViewQuery,
   [TabKind.INDEXES]: TabViewQuery,
   [TabKind.AGGREGATION]: TabViewQuery,
   [TabKind.EXPORTER]: TabViewQuery,
   [TabKind.IMPORTER]: TabViewQuery,
   [TabKind.STATISTICS]: TabViewQuery
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
               )" @click="setActiveTab(id)">
                  <span>{{ tab.context.collection }}</span>
                  <span @click="deleteTab(id, $event)">
                     <CircleX :size="15" class="hover:text-orange-500" />
                  </span>
               </div>
            </template>
         </div>
      </div>
      <div>
         <template v-for="[id] in tabsStore.tabs.entries()" :key="id">
            <div v-show="tabsStore.activeTab === id">
               <KeepAlive>
                  <component :is="components[currentTabType]" :key="tabsStore.activeTab ?? ''" />
               </KeepAlive>
            </div>
         </template>

      </div>
   </div>
</template>