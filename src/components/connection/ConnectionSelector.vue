<script setup lang="ts">
import { computed, ref } from 'vue';
import { ChevronDown, Circle, Loader2, Plus, Server } from '@lucide/vue';
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
import { describeTarget, type Connection } from '@/domains/connections';

const store = useConnectionStore();

const dialogOpen = ref(false);

const openCount = computed(() => store.openConnections.length);
const anyConnecting = computed(() => store.openConnections.some((c) => store.isConnecting(c.id)));

// an open connection is already in the sidebar; picking it again does nothing
function handleSelectConnection(connection: Connection) {
  if (store.sessionOf(connection.id)) return;
  store.connectTo(connection);
}

function handleAddNew() {
  dialogOpen.value = true;
}

</script>

<template>
  <DropdownMenu>
    <DropdownMenuTrigger as-child>
      <Button variant="outline" class="gap-2">
        <Loader2 v-if="anyConnecting" class="w-4 h-4 animate-spin" />
        <Server v-else class="w-4 h-4" />
        <span>Connections</span>
        <span class="rounded bg-secondary px-1.5 text-xs tabular-nums text-muted-foreground">{{ openCount }} open</span>
        <ChevronDown class="w-4 h-4 opacity-50" />
      </Button>
    </DropdownMenuTrigger>

    <DropdownMenuContent align="start" class="w-[280px]">
      <DropdownMenuLabel>Open a connection</DropdownMenuLabel>
      <DropdownMenuSeparator />

      <DropdownMenuItem
        v-for="connection in store.connections"
        :key="connection.id"
        class="gap-2 cursor-pointer"
        :disabled="store.sessionOf(connection.id) !== null"
        @click="handleSelectConnection(connection)"
      >
        <Loader2 v-if="store.isConnecting(connection.id)" class="w-3.5 h-3.5 animate-spin" />
        <Circle
          v-else
          class="w-2.5 h-2.5"
          :class="store.isConnected(connection.id) ? 'fill-success text-success' : 'text-muted-foreground/50'"
        />
        <div class="min-w-0">
          <div class="truncate">{{ connection.name }}</div>
          <div class="truncate text-xs text-muted-foreground">{{ describeTarget(connection.target) }}</div>
        </div>
      </DropdownMenuItem>

      <DropdownMenuSeparator v-if="store.connections.length > 0" />

      <DropdownMenuItem class="gap-2 cursor-pointer" @click="handleAddNew">
        <Plus class="w-4 h-4" />
        <span>Add new connection...</span>
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>

  <ConnectionDialog
    v-model:open="dialogOpen"
    mode="create"
  />
</template>
