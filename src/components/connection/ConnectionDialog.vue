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
import { Eye, EyeOff } from 'lucide-vue-next';

import { useConnectionStore } from '@/stores/connectionStore';
import { 
  AuthenticationKind, 
  ConnectionType, 
  ConnectorKind,
  type Connection 
} from '@/domains/connections';
import { useLogger } from '@/composables/useLogger';

const props = defineProps<{
  open: boolean;
  mode: 'create' | 'edit';
  connection?: Connection;
}>();

const emit = defineEmits<{
  'update:open': [value: boolean];
  'saved': [connection: Connection];
}>();


const logger = useLogger("ConnectionDialog");
const store = useConnectionStore();

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

watch(() => props.connection, async (conn) => {
  if (conn) {
    const useAuth = conn.authentication.kind !== AuthenticationKind.NONE;
    let password = '';
    if (useAuth && props.mode === 'edit') {
      password = (await store.getConnectionPassword(conn.id)) ?? '';
    }
    form.setValues({
      name: conn.name,
      uri: conn.uri,
      port: conn.port,
      useAuth,
      username: conn.authentication.username || '',
      password,
      authDatabase: conn.authentication.database || 'admin',
    });
  }
}, { immediate: true });

watch(isOpen, (open) => {
  if (!open) {
    form.resetForm();
    store.clearTestState();
    showPassword.value = false;
  }
});

const showPassword = ref(false);

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
  logger.debug('handleTest called');
  logger.trace('form values:', JSON.stringify(form.values, null, 2));
  const valid = await form.validate();
  logger.trace('validation result:', valid);
  logger.trace('validation errors:', form.errors.value);
  if (!valid.valid) {
    logger.warn('validation failed, returning early');
    return;
  }

  const connection = buildConnectionFromForm(form.values);
  logger.trace('testing connection:', JSON.stringify(connection, null, 2));
  
  const stages = await store.testConnection(connection);
  logger.trace('test stages:', stages);
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
const everythingFilled = computed(() => {
  if (form.values.useAuth) {
    return form.values.name && form.values.uri && form.values.port && form.values.username && form.values.password && form.values.authDatabase;
  } else {
    return form.values.name && form.values.uri && form.values.port;
  }
});
</script>

<template>
  <Dialog v-model:open="isOpen">
    <DialogContent class="sm:max-w-[700px]">
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
        
        <div class="grid grid-cols-3 gap-4">
          <FormField v-slot="{ componentField }" name="uri">
            <FormItem class="col-span-2">
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
        </div>
        
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
        
        <div v-if="useAuth" class="pl-4 border-l-2 border-muted">
          <div class="grid grid-cols-3 gap-4">
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
                  <div class="relative">
                    <Input 
                      :type="showPassword ? 'text' : 'password'" 
                      placeholder="Password" 
                      v-bind="componentField" 
                      class="pr-10"
                    />
                    <button
                      type="button"
                      class="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
                      tabindex="-1"
                      @click="showPassword = !showPassword"
                    >
                      <EyeOff v-if="showPassword" class="h-4 w-4" />
                      <Eye v-else class="h-4 w-4" />
                    </button>
                  </div>
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
        </div>
        
        <div v-if="store.error" class="p-3 bg-destructive/10 border border-destructive/20 rounded-md">
          <p class="text-sm text-destructive">{{ store.error }}</p>
        </div>

        <div v-if="store.testStages.length > 0 || store.isTesting || store.testError" class="pt-4 border-t">
          <h4 class="text-sm font-medium mb-2">Connection Test</h4>
          <TestStagesDisplay 
            :stages="store.testStages" 
            :error="store.testError"
            :error-detail="store.testErrorDetail"
            :is-loading="store.isTesting"
          />
        </div>
      </form>
      
      <DialogFooter class="gap-2">
        <Button 
          v-if="store.isTesting"
          type="button" 
          variant="outline"
          class="hover:bg-warning/10 hover:text-warning hover:border-warning/50 transition-colors"
          @click="store.cancelTest()"
        >
          Cancel Test
        </Button>
        <Button 
          v-else
          type="button" 
          variant="outline" 
          :disabled="!everythingFilled"
          class="hover:bg-info/10 hover:text-info hover:border-info/50 transition-colors"
          @click="handleTest"
        >
          Test Connection
        </Button>
        <Button 
          type="button" 
          :disabled="!everythingFilled || store.isTesting"
          class="hover:bg-primary/10 hover:text-primary hover:border-primary/50 transition-colors"
          @click="handleSave"
        >
          {{ mode === 'create' ? 'Create' : 'Save' }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
