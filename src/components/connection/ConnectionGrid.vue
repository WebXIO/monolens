<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Database, Plus, Pencil, Trash2, Loader2 } from 'lucide-vue-next';
import Button from '@/components/ui/button/Button.vue';
import ConnectionDialog from './ConnectionDialog.vue';
import { useConnectionStore } from '@/stores/connectionStore';
import type { Connection } from '@/domains/connections';
import { useLogger } from '@/composables/useLogger';

const logger = useLogger("ConnectionGrid");
const store = useConnectionStore();

const dialogOpen = ref(false);
const dialogMode = ref<'create' | 'edit'>('create');
const editingConnection = ref<Connection | undefined>();

const connectingId = ref<string | null>(null);

onMounted(() => {
  store.loadConnections();
});

function openCreateDialog() {
  logger.debug('openCreateDialog called');
  dialogMode.value = 'create';
  editingConnection.value = undefined;
  dialogOpen.value = true;
  logger.debug('dialogOpen set to:', dialogOpen.value);
}

function openEditDialog(connection: Connection) {
  dialogMode.value = 'edit';
  editingConnection.value = connection;
  dialogOpen.value = true;
}

async function handleConnect(connection: Connection) {
  connectingId.value = connection.id;
  await store.connectTo(connection);
  connectingId.value = null;
}

function handleCancelConnect() {
  store.cancelConnect();
  connectingId.value = null;
}

async function handleDelete(connection: Connection) {
  if (confirm(`Delete connection "${connection.name}"?`)) {
    await store.deleteConnection(connection.id);
  }
}

function handleSaved(_connection: Connection) {
  store.loadConnections();
}
</script>
<template>
  <div class="flex flex-col items-center justify-center min-h-[60vh] p-8">
    <div class="text-center mb-8">
      <Database class="w-16 h-16 mx-auto mb-4 text-muted-foreground" />
      <h1 class="text-2xl font-semibold mb-2">MongoDB Connections</h1>
      <p class="text-muted-foreground">
        {{ store.hasConnections ? 'Select a connection to get started' : 'Create your first connection' }}
      </p>
    </div>
    
    <div v-if="store.isLoading" class="flex items-center gap-2 text-muted-foreground">
      <Loader2 class="w-5 h-5 animate-spin" />
      <span>Loading connections...</span>
    </div>
    
    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 w-full max-w-4xl">
      <div
        v-for="connection in store.connections"
        :key="connection.id"
        class="group relative p-4 border rounded-lg bg-card hover:border-primary/50 transition-colors"
      >
        <div class="flex items-start justify-between">
          <div class="flex-1 min-w-0">
            <h3 class="font-medium truncate">{{ connection.name }}</h3>
            <p class="text-sm text-muted-foreground truncate">
              {{ connection.uri }}:{{ connection.port }}
            </p>
          </div>
          
          <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <Button 
              variant="ghost" 
              size="icon"
              class="h-8 w-8"
              @click.stop="openEditDialog(connection)"
            >
              <Pencil class="w-4 h-4" />
            </Button>
            <Button 
              variant="ghost" 
              size="icon"
              class="h-8 w-8 hover:bg-red-600 hover:text-white"
              @click.stop="handleDelete(connection)"
            >
              <Trash2 class="w-4 h-4" />
            </Button>
          </div>
        </div>
        
        <Button 
          v-if="connectingId === connection.id"
          class="w-full mt-3 border-destructive text-destructive hover:bg-destructive/10"
          variant="outline"
          @click.stop="handleCancelConnect"
        >
          <Loader2 class="w-4 h-4 mr-2 animate-spin" />
          Cancel
        </Button>
        <Button 
          v-else
          class="w-full mt-3"
          :disabled="connectingId !== null"
          @click="handleConnect(connection)"
        >
          Connect
        </Button>
      </div>
      
      <button
        class="p-4 border-2 border-dashed rounded-lg flex flex-col items-center justify-center gap-2 text-muted-foreground hover:text-foreground hover:border-primary/50 transition-colors min-h-[120px]"
        @click="openCreateDialog"
      >
        <Plus class="w-8 h-8" />
        <span class="font-medium">Add Connection</span>
      </button>
    </div>
    
    <div 
      v-if="store.error" 
      class="mt-4 p-4 bg-destructive/10 border border-destructive/20 rounded-lg max-w-md"
    >
      <p class="text-sm text-destructive">{{ store.error }}</p>
    </div>
    
    <ConnectionDialog
      v-model:open="dialogOpen"
      :mode="dialogMode"
      :connection="editingConnection"
      @saved="handleSaved"
    />
  </div>
</template>
