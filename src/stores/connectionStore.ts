import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Connection, TestStage } from '@/domains/connections';
import { useLogger } from '@/composables/useLogger';


const _logger = useLogger("ConnectionStore");
const _connections = ref<Connection[]>([]);
const _activeConnectionId = ref<string | null>(null);
const _isLoading = ref(false);
const _error = ref<string | null>(null);

const _testStages = ref<TestStage[]>([]);
const _isTesting = ref(false);
const _testError = ref<string | null>(null);

const _databases = ref<string[]>([]);
const _isDatabasesLoading = ref(false);

const _activeConnection = computed(() => 
  _connections.value.find(c => c.id === _activeConnectionId.value) || null
);

const _hasConnections = computed(() => _connections.value.length > 0);

async function loadConnections() {
  _isLoading.value = true;
  _error.value = null;
  
  try {
    const result = await invoke<Connection[]>('get_connections');
    _connections.value = result;
  } catch (e) {
    _error.value = e instanceof Error ? e.message : String(e);
  } finally {
    _isLoading.value = false;
  }
}

async function createConnection(connection: Omit<Connection, 'id'>): Promise<Connection | null> {
  try {
    const created = await invoke<Connection>('create_connection', { 
      connection: { ...connection, id: '' } 
    });
    _connections.value.push(created);
    return created;
  } catch (e) {
    _error.value = e instanceof Error ? e.message : String(e);
    return null;
  }
}

async function updateConnection(id: string, connection: Connection): Promise<boolean> {
  try {
    await invoke('update_connection', { id, connection });
    const index = _connections.value.findIndex(c => c.id === id);
    if (index !== -1) {
      _connections.value[index] = connection;
    }
    return true;
  } catch (e) {
    _error.value = e instanceof Error ? e.message : String(e);
    return false;
  }
}

async function deleteConnection(id: string): Promise<boolean> {
  try {
    await invoke('delete_connection', { id });
    _connections.value = _connections.value.filter(c => c.id !== id);
    if (_activeConnectionId.value === id) {
      _activeConnectionId.value = null;
      _databases.value = [];
    }
    return true;
  } catch (e) {
    _error.value = e instanceof Error ? e.message : String(e);
    return false;
  }
}

async function testConnection(connection: Omit<Connection, 'id'>): Promise<TestStage[]> {
  _logger.debug('testConnection called with:', connection);
  _isTesting.value = true;
  _testError.value = null;
  _testStages.value = [];
  
  try {
    _logger.trace('invoking test_connection command...');
    const stages = await invoke<TestStage[]>('test_connection', { 
      connection: { ...connection, id: '' } 
    });
    _logger.trace('received stages:', stages);
    _testStages.value = stages;
    return stages;
  } catch (e) {
    _logger.error('test_connection error:', e);
    if (e instanceof Error) {
      _testError.value = e.message;
    } else if (typeof e === 'object' && e !== null) {
      const values = Object.values(e as Record<string, unknown>);
      _testError.value = values.length > 0 ? String(values[0]) : JSON.stringify(e);
    } else {
      _testError.value = String(e);
    }
    return [];
  } finally {
    _isTesting.value = false;
  }
}

async function connectTo(connection: Connection): Promise<boolean> {
  _isDatabasesLoading.value = true;
  
  try {
    const dbs = await invoke<string[]>('get_databases', { connection });
    _databases.value = dbs;
    _activeConnectionId.value = connection.id;
    return true;
  } catch (e) {
    _error.value = e instanceof Error ? e.message : String(e);
    return false;
  } finally {
    _isDatabasesLoading.value = false;
  }
}

function disconnect() {
  _activeConnectionId.value = null;
  _databases.value = [];
}

function clearTestState() {
  _testStages.value = [];
  _testError.value = null;
  _isTesting.value = false;
}

export function useConnectionStore() {
  return {
    connections: _connections,
    activeConnectionId: _activeConnectionId,
    activeConnection: _activeConnection,
    isLoading: _isLoading,
    error: _error,
    hasConnections: _hasConnections,
    
    testStages: _testStages,
    isTesting: _isTesting,
    testError: _testError,
    
    databases: _databases,
    isDatabasesLoading: _isDatabasesLoading,
    
    loadConnections,
    createConnection,
    updateConnection,
    deleteConnection,
    testConnection,
    connectTo,
    disconnect,
    clearTestState,
  };
}
