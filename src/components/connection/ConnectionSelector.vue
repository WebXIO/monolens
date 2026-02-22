<script setup lang="ts">
import { ref, computed } from 'vue';
import { Database, ChevronDown, Plus, Check, Loader2, X } from 'lucide-vue-next';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import Button from '@/components/ui/button/Button.vue';
import ConnectionDialog from './ConnectionDialog.vue';
import { useConnectionStore } from '@/stores/connectionStore';
import type { Connection } from '@/domains/connections';

const store = useConnectionStore();

const dialogOpen = ref(false);
const connectingName = ref<string | null>(null);

const activeConnectionName = computed(() => 
  store.activeConnection?.name || 'No Connection'
);

const displayName = computed(() => {
  if (store.isDatabasesLoading && connectingName.value) {
    return connectingName.value;
  }
  return activeConnectionName.value;
});

async function handleSelectConnection(connection: Connection) {
  if (connection.id === store.activeConnectionId) return;
  connectingName.value = connection.name;
  await store.connectTo(connection);
  connectingName.value = null;
}

function handleCancelConnect() {
  store.cancelConnect();
  connectingName.value = null;
}

function handleAddNew() {
  dialogOpen.value = true;
}

function handleSaved(_connection: Connection) {
  store.loadConnections();
}
</script>

<template>
  <div class="flex items-center gap-1">
    <DropdownMenu>
      <DropdownMenuTrigger as-child>
        <Button variant="outline" class="gap-2" :disabled="store.isDatabasesLoading">
          <Loader2 v-if="store.isDatabasesLoading" class="w-4 h-4 animate-spin" />
          <Database v-else class="w-4 h-4" />
          <span class="max-w-[150px] truncate">{{ displayName }}</span>
          <ChevronDown class="w-4 h-4 opacity-50" />
        </Button>
      </DropdownMenuTrigger>
      
      <DropdownMenuContent align="start" class="w-[220px]">
        <DropdownMenuLabel>Connections</DropdownMenuLabel>
        <DropdownMenuSeparator />
        
        <DropdownMenuItem
          v-for="connection in store.connections"
          :key="connection.id"
          class="gap-2 cursor-pointer"
          :disabled="store.isDatabasesLoading"
          @click="handleSelectConnection(connection)"
        >
          <Check 
            v-if="connection.id === store.activeConnectionId"
            class="w-4 h-4" 
          />
          <div v-else class="w-4 h-4" />
          <span class="truncate">{{ connection.name }}</span>
        </DropdownMenuItem>
        
        <DropdownMenuSeparator v-if="store.connections.length > 0" />
        
        <DropdownMenuItem class="gap-2 cursor-pointer" @click="handleAddNew">
          <Plus class="w-4 h-4" />
          <span>Add new connection...</span>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>

    <TooltipProvider v-if="store.isDatabasesLoading">
      <Tooltip>
        <TooltipTrigger as-child>
          <Button
            variant="ghost"
            size="icon"
            class="h-8 w-8 hover:bg-destructive/10 hover:text-destructive"
            @click="handleCancelConnect"
          >
            <X class="w-4 h-4" />
          </Button>
        </TooltipTrigger>
        <TooltipContent side="bottom" class="text-xs">
          Cancel connection
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  </div>
  
  <ConnectionDialog
    v-model:open="dialogOpen"
    mode="create"
    @saved="handleSaved"
  />
</template>
