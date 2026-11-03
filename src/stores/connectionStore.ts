import { useLogger } from '@/composables/useLogger';
import { TaskCancelledError } from '@/composables/useAbortableCommand';
import { useDomain } from '@/domains';
import { Connection, ConnectionServiceIPC, TestStage } from '@/domains/connections';
import { extractErrorMessage } from '@/utils/errorMessage';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { toast } from 'vue-sonner';
import { useTabsStore } from './tabsStore';

// this is how the payload from Rust looks like for progress events
interface TestProgressPayload {
  testId: string;
  stageIndex: number;
  stage: TestStage;
}

/** An open connection: the Rust core holds its driver, this its databases and collections. */
export interface ConnectionSession {
  status: 'connecting' | 'connected';
  databases: string[];
  /** collection names per database, loaded when a database is expanded */
  collections: Record<string, string[]>;
  /** busy re-listing the databases */
  refreshing: boolean;
}

export const useConnectionStore = defineStore('connection', () => {
  const logger = useLogger("ConnectionStore");
  const connectionService = useDomain<ConnectionServiceIPC>(ConnectionServiceIPC);

  const connections = ref<Connection[]>([]);
  const sessions = ref<Record<string, ConnectionSession>>({});
  /** connection ids in the order they were opened, for the sidebar */
  const openOrder = ref<string[]>([]);
  const hasConnections = computed(() => connections.value.length > 0);
  const openConnections = computed(() =>
    openOrder.value
      .map((id) => connections.value.find((c) => c.id === id))
      .filter((c): c is Connection => c !== undefined),
  );
  // the grid gives way only once a connection works, not while it connects (issue #27)
  const hasOpenConnections = computed(() => Object.values(sessions.value).some((session) => session.status === 'connected'));
  const isTesting = ref(false);
  const testStages = ref<TestStage[]>([]);

  const isLoading = ref(false);

  const testAbortController = ref<AbortController | null>(null);
  const connectAbortControllers = new Map<string, AbortController>();

  function getConnection(id: string): Connection | null {
    return connections.value.find((c) => c.id === id) ?? null;
  }

  function sessionOf(id: string): ConnectionSession | null {
    return sessions.value[id] ?? null;
  }

  function isConnecting(id: string): boolean {
    return sessions.value[id]?.status === 'connecting';
  }

  function isConnected(id: string): boolean {
    return sessions.value[id]?.status === 'connected';
  }

  async function loadConnections() {
    isLoading.value = true;

    const result = await connectionService.listConnections();
    if (result.data) {
      connections.value = result.data;
    } else {
      toast.error(extractErrorMessage(result.error));
    }
    isLoading.value = false;
  }

  async function createConnection(connection: Omit<Connection, 'id'>): Promise<Connection | null> {
    const created = await connectionService.createConnection(connection);
    if (created.data) {
      connections.value.push(created.data);
      return created.data;
    } else {
      toast.error(extractErrorMessage(created.error));
      return null;
    }
  }

  async function updateConnection(id: string, connection: Connection): Promise<boolean> {
    const result = await connectionService.updateConnection(id, connection);
    if (result.error) {
      toast.error(extractErrorMessage(result.error));
      return false;
    }
    const index = connections.value.findIndex(c => c.id === id);
    if (index === -1) {
      toast.error(`Connection with id ${id} not found.`);
      return false;
    }
    // the stored connection never holds the password
    connections.value[index] = { ...connection, authentication: { ...connection.authentication, password: null } };

    // the Rust core dropped the driver; an open connection reconnects with the new settings
    if (isConnected(id)) {
      await refresh(id);
    }
    return true;
  }

  async function deleteConnection(id: string): Promise<boolean> {
    if (sessions.value[id]) {
      await disconnect(id);
    }
    const result = await connectionService.deleteConnection(id);
    if (result.error) {
      toast.error(extractErrorMessage(result.error));
      return false;
    }
    connections.value = connections.value.filter(c => c.id !== id);
    return true;
  }

  function closeSession(id: string) {
    const { [id]: _closed, ...rest } = sessions.value;
    sessions.value = rest;
    openOrder.value = openOrder.value.filter((openId) => openId !== id);
  }

  /** Opens a connection next to the ones already open; an open one stays as it is. */
  async function connectTo(connection: Connection): Promise<boolean> {
    if (sessions.value[connection.id]) return true;

    sessions.value = {
      ...sessions.value,
      [connection.id]: { status: 'connecting', databases: [], collections: {}, refreshing: false },
    };
    openOrder.value = [...openOrder.value, connection.id];

    const controller = new AbortController();
    connectAbortControllers.set(connection.id, controller);

    try {
      const databases = await connectionService.connect(connection.id, { signal: controller.signal });
      const session = sessions.value[connection.id];
      if (!session) return false;
      session.databases = databases;
      session.status = 'connected';
      return true;
    } catch (e) {
      closeSession(connection.id);
      if (e instanceof TaskCancelledError) {
        logger.info('connectTo was cancelled by user');
        return false;
      }
      toast.error(extractErrorMessage(e));
      return false;
    } finally {
      connectAbortControllers.delete(connection.id);
    }
  }

  function cancelConnect(id: string) {
    connectAbortControllers.get(id)?.abort();
  }

  /** Lists the databases again and forgets the cached collections. */
  async function refresh(id: string): Promise<boolean> {
    const session = sessions.value[id];
    if (!session || session.status !== 'connected') return false;

    session.refreshing = true;
    try {
      session.databases = await connectionService.connect(id);
      session.collections = {};
      return true;
    } catch (e) {
      toast.error(extractErrorMessage(e));
      return false;
    } finally {
      session.refreshing = false;
    }
  }

  /** Closes a connection: its tabs close, the Rust core drops its driver. */
  async function disconnect(id: string) {
    cancelConnect(id);
    closeSession(id);
    useTabsStore().closeTabsOf(id);

    const result = await connectionService.disconnect(id);
    if (result.error) {
      logger.error('disconnect failed:', result.error);
    }
  }

  async function disconnectAll() {
    await Promise.all(openOrder.value.map((id) => disconnect(id)));
  }

  /** The collections of a database, cached until the connection is refreshed. */
  async function listCollections(id: string, databaseName: string): Promise<string[] | null> {
    const session = sessions.value[id];
    if (!session) return null;

    const cached = session.collections[databaseName];
    if (cached) return cached;

    const result = await connectionService.listCollections(id, databaseName);
    if (result.error) {
      toast.error(extractErrorMessage(result.error));
      return null;
    }
    // the session may have closed while waiting
    const current = sessions.value[id];
    if (current) current.collections[databaseName] = result.data;
    return result.data;
  }

  async function testConnection(connection: Omit<Connection, 'id'>): Promise<TestStage[]> {
    isTesting.value = true;
    // the driver reports every stage as pending first, then each result
    testStages.value = [];
    const testId = globalThis.crypto.randomUUID();

    const controller = new AbortController();
    testAbortController.value = controller;
    let unlisten: UnlistenFn | null = null;

    try {
      unlisten = await listen<TestProgressPayload>('test-connection-progress', (event) => {
        const { testId: eventTestId, stageIndex, stage } = event.payload;
        if (eventTestId !== testId || stageIndex < 0) return;
        const next = [...testStages.value];
        next[stageIndex] = { ...stage };
        testStages.value = next;
      });

      const stages = await connectionService.testConnection(connection, testId, { signal: controller.signal });

      // Set final stages from the completed result
      testStages.value = stages;
      return stages;
    } catch (e) {
      if (e instanceof TaskCancelledError) {
        logger.info('testConnection was cancelled by user');
        toast.info('Test cancelled.');
        return [];
      }
      logger.error('test_connection error:', e);
      toast.error(extractErrorMessage(e));
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

  function clearTestState() {
    testStages.value = [];
    isTesting.value = false;
  }

  async function getConnectionPassword(id: string): Promise<string | null> {
    const result = await connectionService.getConnectionPassword(id);
    if (result.data !== undefined && result.data !== null) {
      return result.data;
    }
    return null;
  }

  return {
    createConnection,
    updateConnection,
    deleteConnection,
    loadConnections,
    connectTo,
    cancelConnect,
    refresh,
    disconnect,
    disconnectAll,
    listCollections,
    testConnection,
    cancelTest,
    clearTestState,
    getConnectionPassword,
    getConnection,
    sessionOf,
    isConnecting,
    isConnected,

    connections,
    sessions,
    openConnections,
    hasConnections,
    hasOpenConnections,
    isLoading,
    testStages,
    isTesting,
  }
});
