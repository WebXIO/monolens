<script setup lang="ts">
import { ref, computed } from 'vue';
import { Database, ChevronDown, Plus, Check } from 'lucide-vue-next';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import Button from '@/components/ui/button/Button.vue';
import ConnectionDialog from './ConnectionDialog.vue';
import { useConnectionStore } from '@/stores/connectionStore';
import type { Connection } from '@/domains/connections';

const store = useConnectionStore();

const connections = computed(() => store.connections.value);
const activeConnectionId = computed(() => store.activeConnectionId.value);
const activeConnection = computed(() => store.activeConnection.value);

const dialogOpen = ref(false);

const activeConnectionName = computed(() => 
  activeConnection.value?.name || 'No Connection'
);

async function handleSelectConnection(connection: Connection) {
  if (connection.id === activeConnectionId.value) return;
  await store.connectTo(connection);
}

function handleAddNew() {
  dialogOpen.value = true;
}

function handleSaved(_connection: Connection) {
  store.loadConnections();
}
</script>

<template>
  <DropdownMenu>
    <DropdownMenuTrigger as-child>
      <Button variant="outline" class="gap-2">
        <Database class="w-4 h-4" />
        <span class="max-w-[150px] truncate">{{ activeConnectionName }}</span>
        <ChevronDown class="w-4 h-4 opacity-50" />
      </Button>
    </DropdownMenuTrigger>
    
    <DropdownMenuContent align="start" class="w-[220px]">
      <DropdownMenuLabel>Connections</DropdownMenuLabel>
      <DropdownMenuSeparator />
      
      <DropdownMenuItem
        v-for="connection in connections"
        :key="connection.id"
        class="gap-2 cursor-pointer"
        @click="handleSelectConnection(connection)"
      >
        <Check 
          v-if="connection.id === activeConnectionId"
          class="w-4 h-4" 
        />
        <div v-else class="w-4 h-4" />
        <span class="truncate">{{ connection.name }}</span>
      </DropdownMenuItem>
      
      <DropdownMenuSeparator v-if="connections.length > 0" />
      
      <DropdownMenuItem class="gap-2 cursor-pointer" @click="handleAddNew">
        <Plus class="w-4 h-4" />
        <span>Add new connection...</span>
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
  
  <ConnectionDialog
    v-model:open="dialogOpen"
    mode="create"
    @saved="handleSaved"
  />
</template>
