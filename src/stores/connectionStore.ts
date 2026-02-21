import { useLogger } from '@/composables/useLogger';
import { TaskCancelledError } from '@/composables/useAbortableCommand';
import { useDomain } from '@/domains';
import { Connection, ConnectionServiceIPC, TestStage } from '@/domains/connections';
import { extractErrorMessage } from '@/utils/errorMessage';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { defineStore } from 'pinia';
import { computed, ref } from 'vue';

// this is how the payload from Rust looks like for progress events
interface TestProgressPayload {
  taskId: string;
  stageIndex: number;
  stage: TestStage;
}

export const useConnectionStore = defineStore('connection', () => {
  const logger = useLogger("ConnectionStore");
  const connectionService = useDomain<ConnectionServiceIPC>(ConnectionServiceIPC);

  const connections = ref<Connection[]>([]);
  const activeConnectionId = ref<string | null>(null);
  const activeConnection = computed(() => connections.value.find(c => c.id === activeConnectionId.value) || null);
  const hasConnections = computed(() => connections.value.length > 0);
  const databases = ref<string[]>([]);
  const isTesting = ref(false);
  const testStages = ref<TestStage[]>([]);
  const testError = ref<string | null>(null);
  const testErrorDetail = ref<string | null>(null);

  const isLoading = ref(false);
  const error = ref<string | null>(null);
  const isDatabasesLoading = ref(false);

  const testAbortController = ref<AbortController | null>(null);
  const connectAbortController = ref<AbortController | null>(null);

  async function loadConnections() {
    isLoading.value = true;
    error.value = null;

    const result = await connectionService.listConnections();
    if (result.data) {
      connections.value = result.data;
    } else {
      error.value = extractErrorMessage(result.error);
    }
    isLoading.value = false;
  }

  async function createConnection(connection: Omit<Connection, 'id'>): Promise<Connection | null> {
    const created = await connectionService.createConnection(connection);
    if (created.data) {
      connections.value.push(created.data);
      return created.data;
    } else {
      error.value = extractErrorMessage(created.error);
      return null;
    }
  }

  async function updateConnection(id: string, connection: Connection): Promise<boolean> {
    const result = await connectionService.updateConnection(id, connection);
    if (result.error) {
      error.value = extractErrorMessage(result.error);
      return false;
    }
    const index = connections.value.findIndex(c => c.id === id);
    if (index !== -1) {
      connections.value[index] = connection;
      return true;
    } else {
      error.value = `Connection with id ${id} not found.`;
      return false;
    }
  }

  async function deleteConnection(id: string): Promise<boolean> {
    const result = await connectionService.deleteConnection(id);
    if (result.error) {
      error.value = extractErrorMessage(result.error);
      return false;
    }
    connections.value = connections.value.filter(c => c.id !== id);
    if (activeConnectionId.value === id) {
        activeConnectionId.value = null;
        databases.value = [];
    }
    return true;
  }

  async function connectTo(connection: Connection): Promise<boolean> {
    isDatabasesLoading.value = true;
    error.value = null;

    const controller = new AbortController();
    connectAbortController.value = controller;

    try {
      const dbs = await connectionService.connect(connection, { signal: controller.signal });
      databases.value = dbs;
      activeConnectionId.value = connection.id;
      return true;
    } catch (e) {
      if (e instanceof TaskCancelledError) {
        logger.info('connectTo was cancelled by user');
        error.value = null;
        return false;
      }
      error.value = extractErrorMessage(e);
      return false;
    } finally {
      connectAbortController.value = null;
      isDatabasesLoading.value = false;
    }
  }

  function cancelConnect() {
    connectAbortController.value?.abort();
  }

  async function testConnection(connection: Omit<Connection, 'id'>): Promise<TestStage[]> {
    isTesting.value = true;
    testError.value = null;
    testErrorDetail.value = null;
    testStages.value = _createPendingStages();

    const controller = new AbortController();
    testAbortController.value = controller;
    let unlisten: UnlistenFn | null = null;

    try {
      unlisten = await listen<TestProgressPayload>('test-connection-progress', (event) => {
        const { stageIndex, stage } = event.payload;
        if (stageIndex >= 0 && stageIndex < testStages.value.length) {
          testStages.value[stageIndex] = { ...stage };
        }
      });

      const stages = await connectionService.testConnection(connection, { signal: controller.signal });

      // Set final stages from the completed result
      testStages.value = stages;
      return stages;
    } catch (e) {
      if (e instanceof TaskCancelledError) {
        logger.info('testConnection was cancelled by user');
        testError.value = 'Test cancelled.';
        return [];
      }
      logger.error('test_connection error:', e);
      testError.value = extractErrorMessage(e);
      if (typeof e === 'object' && e !== null && 'message' in e) {
        testErrorDetail.value = (e as Record<string, unknown>).message as string;
      }
      return [];
    } finally {
      unlisten?.();
      testAbortController.value = null;
      isTesting.value = false;
    }
  }

  function cancelTest() {
    testAbortController.value?.abort();
  }

  function disconnect() {
    activeConnectionId.value = null;
    databases.value = [];
  }

  function clearTestState() {
    testStages.value = [];
    testError.value = null;
    testErrorDetail.value = null;
    isTesting.value = false;
  }

  async function getConnectionPassword(id: string): Promise<string | null> {
    const result = await connectionService.getConnectionPassword(id);
    if (result.data !== undefined && result.data !== null) {
      return result.data;
    }
    return null;
  }

  function _createPendingStages(): TestStage[] {
    return [
      { title: 'Initialize Connection', status: null },
      { title: 'Ping Database', status: null },
      { title: 'Reading Server status', status: null },
      { title: 'Detecting Mongodb version', status: null },
      { title: 'Connected', status: null },
    ]
  }

  return {
    createConnection,
    updateConnection,
    deleteConnection,
    loadConnections,
    connectTo,
    cancelConnect,
    disconnect,
    testConnection,
    cancelTest,
    clearTestState,
    getConnectionPassword,

    connections,
    activeConnection,
    activeConnectionId,
    isLoading,
    error,
    hasConnections,
    testStages,
    isTesting,
    testError,
    testErrorDetail,
    databases,
    isDatabasesLoading,
  }
});
