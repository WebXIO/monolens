<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { toTypedSchema } from "@vee-validate/zod";
import * as z from "zod";
import { useForm } from "vee-validate";

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import Input from '@/components/ui/input/Input.vue';
import Button from '@/components/ui/button/Button.vue';
import TestStagesDisplay from './TestStagesDisplay.vue';

import { useConnectionStore } from '@/stores/connectionStore';
import { 
  AuthenticationKind, 
  ConnectionType, 
  ConnectorKind,
  type Connection 
} from '@/domains/connections';

const props = defineProps<{
  open: boolean;
  mode: 'create' | 'edit';
  connection?: Connection;
}>();

const emit = defineEmits<{
  'update:open': [value: boolean];
  'saved': [connection: Connection];
}>();

const store = useConnectionStore();

const testStages = computed(() => store.testStages.value);
const isTesting = computed(() => store.isTesting.value);
const testError = computed(() => store.testError.value);

const isOpen = computed({
  get: () => props.open,
  set: (value) => emit('update:open', value),
});

const dialogTitle = computed(() => 
  props.mode === 'create' ? 'New Connection' : 'Edit Connection'
);

const dialogDescription = computed(() =>
  props.mode === 'create' 
    ? 'Add a new MongoDB connection' 
    : 'Update connection settings'
);

const formSchema = toTypedSchema(z.object({
  name: z.string().min(1, 'Name is required'),
  uri: z.string().min(1, 'URI is required'),
  port: z.coerce.number().min(1).max(65535).default(27017),
  useAuth: z.boolean().default(false),
  username: z.string().optional(),
  password: z.string().optional(),
  authDatabase: z.string().optional(),
}));

const form = useForm({
  validationSchema: formSchema,
  initialValues: {
    name: '',
    uri: 'localhost',
    port: 27017,
    useAuth: false,
    username: '',
    password: '',
    authDatabase: 'admin',
  },
});

watch(() => props.connection, (conn) => {
  if (conn) {
    form.setValues({
      name: conn.name,
      uri: conn.uri,
      port: conn.port,
      useAuth: conn.authentication.kind !== AuthenticationKind.NONE,
      username: conn.authentication.username || '',
      password: conn.authentication.password || '',
      authDatabase: conn.authentication.database || 'admin',
    });
  }
}, { immediate: true });

watch(isOpen, (open) => {
  if (!open) {
    form.resetForm();
    store.clearTestState();
    testPassed.value = false;
  }
});

const testPassed = ref(false);

function buildConnectionFromForm(values: typeof form.values): Omit<Connection, 'id'> {
  return {
    name: values.name || '',
    uri: values.uri || '',
    port: values.port || 27017,
    connector: ConnectorKind.MongoDb,
    connectionType: ConnectionType.Standalone,
    authentication: {
      kind: values.useAuth ? AuthenticationKind.BASIC : AuthenticationKind.NONE,
      username: values.useAuth && values.username ? values.username : null,
      password: values.useAuth && values.password ? values.password : null,
      database: values.useAuth && values.authDatabase ? values.authDatabase : null,
    },
  };
}

async function handleTest() {
  console.log('[ConnectionDialog] handleTest called');
  console.log('[ConnectionDialog] form values:', JSON.stringify(form.values, null, 2));
  const valid = await form.validate();
  console.log('[ConnectionDialog] validation result:', valid);
  console.log('[ConnectionDialog] validation errors:', form.errors.value);
  if (!valid.valid) {
    console.log('[ConnectionDialog] validation failed, returning early');
    return;
  }

  testPassed.value = false;
  const connection = buildConnectionFromForm(form.values);
  console.log('[ConnectionDialog] testing connection:', JSON.stringify(connection, null, 2));
  
  const stages = await store.testConnection(connection);
  console.log('[ConnectionDialog] test stages:', stages);
  
  testPassed.value = stages.length > 0 && stages.every(s => s.status === true);
}

async function handleSave() {
  const valid = await form.validate();
  if (!valid.valid) return;

  const connectionData = buildConnectionFromForm(form.values);
  
  let saved: Connection | null = null;
  
  if (props.mode === 'create') {
    saved = await store.createConnection(connectionData);
  } else if (props.connection) {
    const updated = { ...connectionData, id: props.connection.id };
    const success = await store.updateConnection(props.connection.id, updated as Connection);
    if (success) {
      saved = updated as Connection;
    }
  }
  
  if (saved) {
    emit('saved', saved);
    isOpen.value = false;
  }
}

const useAuth = computed(() => form.values.useAuth);
</script>

<template>
  <Dialog v-model:open="isOpen">
    <DialogContent class="sm:max-w-[500px]">
      <DialogHeader>
        <DialogTitle>{{ dialogTitle }}</DialogTitle>
        <DialogDescription>{{ dialogDescription }}</DialogDescription>
      </DialogHeader>
      
      <form class="space-y-4" @submit.prevent>
        <FormField v-slot="{ componentField }" name="name">
          <FormItem>
            <FormLabel>Connection Name</FormLabel>
            <FormControl>
              <Input 
                type="text" 
                placeholder="My MongoDB" 
                v-bind="componentField" 
              />
            </FormControl>
            <FormMessage />
          </FormItem>
        </FormField>
        
        <FormField v-slot="{ componentField }" name="uri">
          <FormItem>
            <FormLabel>Host</FormLabel>
            <FormControl>
              <Input 
                type="text" 
                placeholder="localhost or mongodb+srv://..." 
                v-bind="componentField" 
              />
            </FormControl>
            <FormMessage />
          </FormItem>
        </FormField>
        
        <FormField v-slot="{ componentField }" name="port">
          <FormItem>
            <FormLabel>Port</FormLabel>
            <FormControl>
              <Input 
                type="number" 
                placeholder="27017" 
                v-bind="componentField" 
              />
            </FormControl>
            <FormMessage />
          </FormItem>
        </FormField>
        
        <FormField v-slot="{ value, handleChange }" name="useAuth">
          <FormItem class="flex items-center gap-3">
            <FormControl>
              <input 
                type="checkbox" 
                :checked="value"
                class="h-4 w-4 rounded border-input"
                @change="(e: Event) => handleChange((e.target as HTMLInputElement).checked)"
              />
            </FormControl>
            <FormLabel class="!mt-0">Use Authentication</FormLabel>
          </FormItem>
        </FormField>
        
        <div v-if="useAuth" class="space-y-4 pl-4 border-l-2 border-muted">
          <FormField v-slot="{ componentField }" name="username">
            <FormItem>
              <FormLabel>Username</FormLabel>
              <FormControl>
                <Input 
                  type="text" 
                  placeholder="Username" 
                  v-bind="componentField" 
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          </FormField>
          
          <FormField v-slot="{ componentField }" name="password">
            <FormItem>
              <FormLabel>Password</FormLabel>
              <FormControl>
                <Input 
                  type="password" 
                  placeholder="Password" 
                  v-bind="componentField" 
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          </FormField>
          
          <FormField v-slot="{ componentField }" name="authDatabase">
            <FormItem>
              <FormLabel>Auth Database</FormLabel>
              <FormControl>
                <Input 
                  type="text" 
                  placeholder="admin" 
                  v-bind="componentField" 
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          </FormField>
        </div>
        
        <div v-if="testStages.length > 0 || isTesting || testError" class="pt-4 border-t">
          <h4 class="text-sm font-medium mb-2">Connection Test</h4>
          <TestStagesDisplay 
            :stages="testStages" 
            :error="testError"
            :is-loading="isTesting"
          />
        </div>
      </form>
      
      <DialogFooter class="gap-2">
        <Button 
          type="button" 
          variant="outline" 
          :disabled="isTesting"
          @click="handleTest"
        >
          {{ isTesting ? 'Testing...' : 'Test Connection' }}
        </Button>
        <Button 
          type="button" 
          :disabled="!testPassed || isTesting"
          @click="handleSave"
        >
          {{ mode === 'create' ? 'Create' : 'Save' }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
