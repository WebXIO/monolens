<script setup lang="ts">
import { useTabsStore } from '@/stores/tabsStore';
import { useConnectionStore } from '@/stores/connectionStore';
import { cn } from '@/lib/utils';
import { CircleX } from '@lucide/vue';
import { TabKind } from '@/domains/tabs';
import TabViewQuery from './views/TabViewQuery.vue';

const tabsStore = useTabsStore();
const connectionStore = useConnectionStore();

function setActiveTab(id: string) {
   tabsStore.switchTab(id);
}

function deleteTab(id: string, event?: MouseEvent) {
   tabsStore.deleteTab(id);
   event?.stopPropagation(); // prevent propagation so that the switch event is not called after deletion
}

function connectionName(connectionId: string) {
   return connectionStore.getConnection(connectionId)?.name ?? '';
}

// only query tabs are built so far; the other kinds fall back to it
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
      <div class="flex shrink-0 items-center border-b border-border bg-sidebar">
         <div class="flex flex-1 items-center overflow-x-auto">
            <template v-for="[id, tab] in tabsStore.tabs.entries()" :key="id">
               <div :class="cn(
                  'group flex cursor-pointer items-center gap-2 border-b-2 px-4 py-1.5 text-sm transition-colors',
                  tabsStore.activeTab === id
                     ? 'border-primary bg-card text-foreground'
                     : 'border-transparent text-muted-foreground hover:text-foreground'
               )" :title="`${connectionName(tab.context.connectionId)} · ${tab.context.database}.${tab.context.collection}`" @click="setActiveTab(id)">
                  <span class="flex flex-col leading-tight">
                     <span>{{ tab.context.collection }}</span>
                     <span class="text-[11px] text-muted-foreground">{{ connectionName(tab.context.connectionId) }} · {{ tab.context.database }}</span>
                  </span>
                  <span aria-label="Close tab" @click="deleteTab(id, $event)">
                     <CircleX :size="15" class="hover:text-orange-500" />
                  </span>
               </div>
            </template>
         </div>
      </div>
      <div class="flex-1 min-h-0 overflow-hidden">
         <!-- one view per tab, kept while hidden so it keeps its query and results -->
         <template v-for="[id, tab] in tabsStore.tabs.entries()" :key="id">
            <div v-show="tabsStore.activeTab === id" class="h-full">
               <component :is="components[tab.type]" :tab-id="id" />
            </div>
         </template>
      </div>
   </div>
</template>
