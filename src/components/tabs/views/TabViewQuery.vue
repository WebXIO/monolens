<script setup lang="ts">
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useDomain } from '@/domains';
import { CommandRegistry, Commands } from '@/domains/shortcuts';
import { useConnectionStore } from '@/stores/connectionStore';
import { useTabsStore } from '@/stores/tabsStore';
import { invoke } from '@tauri-apps/api/core';
import { onMounted, onUnmounted, ref } from 'vue';

const commandHandler = useDomain<CommandRegistry>(CommandRegistry);
const tabsStore = useTabsStore();
const connectionStore = useConnectionStore();

const query = ref<string>('{ }');
const limit = ref<number | undefined>();
const skip = ref<number | undefined>();
const result = ref<any>({});

async function execute() {
   if(!connectionStore.activeConnection) return;
   const tab = tabsStore.getActiveTab(); 
   if(!tab) return;

   const id = await invoke('start_find_documents', {
      connection: connectionStore.activeConnection,
      databaseName: tab.context.database,
      collectionName: tab.context.collection,
      filter: JSON.parse(query.value),
      skip: 0,
      limit: 300,
   })

   result.value = await invoke('await_task_result', { id });
}

onMounted(() => {
   commandHandler.subscribe(Commands.EVENT_EXECUTE, execute);
})

onUnmounted(() => {
   commandHandler.unsubscribe(Commands.EVENT_EXECUTE, execute);
})

</script>

<template>
   <div class="pa-2">
      <Label>Query</Label>
      <Input v-model="query" />

      <div class="border border-b border-primary"></div>
      {{ result }}
   </div>
</template>
