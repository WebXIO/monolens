<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useForm } from '@tanstack/vue-form';
import { z } from 'zod';

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Field, FieldError, FieldLabel } from '@/components/ui/field';
import Input from '@/components/ui/input/Input.vue';
import Button from '@/components/ui/button/Button.vue';
import TestStagesDisplay from './TestStagesDisplay.vue';
import { Eye, EyeOff } from '@lucide/vue';

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

const formSchema = z.object({
  name: z.string().min(1, 'Name is required'),
  uri: z.string().min(1, 'Host is required'),
  port: z.number().int().min(1).max(65535),
  useAuth: z.boolean(),
  username: z.string(),
  password: z.string(),
  authDatabase: z.string(),
});

type FormValues = z.infer<typeof formSchema>;
type SubmitAction = { action: 'test' | 'save' };

const defaultValues: FormValues = {
  name: '',
  uri: 'localhost',
  port: 27017,
  useAuth: false,
  username: '',
  password: '',
  authDatabase: 'admin',
};

const form = useForm({
  defaultValues,
  validators: {
    onSubmit: formSchema,
  },
  onSubmitMeta: { action: 'save' } as SubmitAction,
  onSubmit: async ({ value, meta }) => {
    if (meta.action === 'test') {
      await handleTest(value);
    } else {
      await handleSave(value);
    }
  },
});

const values = form.useSelector((state) => state.values);

function isInvalid(field: { state: { meta: { isTouched: boolean; isValid: boolean } } }) {
  return field.state.meta.isTouched && !field.state.meta.isValid;
}

watch(() => props.connection, async (conn) => {
  if (conn) {
    const useAuth = conn.authentication.kind !== AuthenticationKind.NONE;
    let password = '';
    if (useAuth && props.mode === 'edit') {
      password = (await store.getConnectionPassword(conn.id)) ?? '';
    }
    form.reset({
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
    form.reset(defaultValues);
    store.clearTestState();
    showPassword.value = false;
  }
});

const showPassword = ref(false);

function buildConnectionFromForm(values: FormValues): Omit<Connection, 'id'> {
  return {
    name: values.name,
    uri: values.uri,
    port: values.port,
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

async function handleTest(values: FormValues) {
  logger.debug('testing connection');
  await store.testConnection(buildConnectionFromForm(values));
}

async function handleSave(values: FormValues) {
  const connectionData = buildConnectionFromForm(values);

  let saved: Connection | null = null;

  if (props.mode === 'create') {
    saved = await store.createConnection(connectionData);
  } else if (props.connection) {
    const updated = { ...connectionData, id: props.connection.id };
    const success = await store.updateConnection(props.connection.id, updated);
    if (success) {
      saved = updated;
    }
  }

  if (saved) {
    emit('saved', saved);
    isOpen.value = false;
  }
}

const useAuth = computed(() => values.value.useAuth);
const everythingFilled = computed(() => {
  const v = values.value;
  if (v.useAuth) {
    return Boolean(v.name && v.uri && v.port && v.username && v.password && v.authDatabase);
  }
  return Boolean(v.name && v.uri && v.port);
});
</script>

<template>
  <Dialog v-model:open="isOpen">
    <DialogContent class="sm:max-w-[700px]">
      <DialogHeader>
        <DialogTitle>{{ dialogTitle }}</DialogTitle>
        <DialogDescription>{{ dialogDescription }}</DialogDescription>
      </DialogHeader>

      <form class="space-y-4" @submit.prevent="form.handleSubmit({ action: 'save' })">
        <form.Field name="name">
          <template #default="{ field }">
            <Field :data-invalid="isInvalid(field)">
              <FieldLabel :for="field.name">Connection Name</FieldLabel>
              <Input
                :id="field.name"
                :name="field.name"
                :model-value="field.state.value"
                :aria-invalid="isInvalid(field)"
                type="text"
                placeholder="My MongoDB"
                @blur="field.handleBlur"
                @update:model-value="(value) => field.handleChange(String(value))"
              />
              <FieldError v-if="isInvalid(field)" :errors="field.state.meta.errors" />
            </Field>
          </template>
        </form.Field>

        <div class="grid grid-cols-3 gap-4">
          <form.Field name="uri">
            <template #default="{ field }">
              <Field class="col-span-2" :data-invalid="isInvalid(field)">
                <FieldLabel :for="field.name">Host</FieldLabel>
                <Input
                  :id="field.name"
                  :name="field.name"
                  :model-value="field.state.value"
                  :aria-invalid="isInvalid(field)"
                  type="text"
                  placeholder="localhost"
                  @blur="field.handleBlur"
                  @update:model-value="(value) => field.handleChange(String(value))"
                />
                <FieldError v-if="isInvalid(field)" :errors="field.state.meta.errors" />
              </Field>
            </template>
          </form.Field>

          <form.Field name="port">
            <template #default="{ field }">
              <Field :data-invalid="isInvalid(field)">
                <FieldLabel :for="field.name">Port</FieldLabel>
                <Input
                  :id="field.name"
                  :name="field.name"
                  :model-value="field.state.value"
                  :aria-invalid="isInvalid(field)"
                  type="number"
                  placeholder="27017"
                  @blur="field.handleBlur"
                  @update:model-value="(value) => field.handleChange(Number(value))"
                />
                <FieldError v-if="isInvalid(field)" :errors="field.state.meta.errors" />
              </Field>
            </template>
          </form.Field>
        </div>

        <form.Field name="useAuth">
          <template #default="{ field }">
            <Field orientation="horizontal">
              <input
                :id="field.name"
                :checked="field.state.value"
                type="checkbox"
                class="h-4 w-4 rounded border-input"
                @change="(e: Event) => field.handleChange((e.target as HTMLInputElement).checked)"
              />
              <FieldLabel :for="field.name">Use Authentication</FieldLabel>
            </Field>
          </template>
        </form.Field>

        <div v-if="useAuth" class="pl-4 border-l-2 border-muted">
          <div class="grid grid-cols-3 gap-4">
            <form.Field name="username">
              <template #default="{ field }">
                <Field :data-invalid="isInvalid(field)">
                  <FieldLabel :for="field.name">Username</FieldLabel>
                  <Input
                    :id="field.name"
                    :name="field.name"
                    :model-value="field.state.value"
                    type="text"
                    placeholder="Username"
                    @blur="field.handleBlur"
                    @update:model-value="(value) => field.handleChange(String(value))"
                  />
                  <FieldError v-if="isInvalid(field)" :errors="field.state.meta.errors" />
                </Field>
              </template>
            </form.Field>

            <form.Field name="password">
              <template #default="{ field }">
                <Field :data-invalid="isInvalid(field)">
                  <FieldLabel :for="field.name">Password</FieldLabel>
                  <div class="relative">
                    <Input
                      :id="field.name"
                      :name="field.name"
                      :model-value="field.state.value"
                      :type="showPassword ? 'text' : 'password'"
                      placeholder="Password"
                      class="pr-10"
                      @blur="field.handleBlur"
                      @update:model-value="(value) => field.handleChange(String(value))"
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
                  <FieldError v-if="isInvalid(field)" :errors="field.state.meta.errors" />
                </Field>
              </template>
            </form.Field>

            <form.Field name="authDatabase">
              <template #default="{ field }">
                <Field :data-invalid="isInvalid(field)">
                  <FieldLabel :for="field.name">Auth Database</FieldLabel>
                  <Input
                    :id="field.name"
                    :name="field.name"
                    :model-value="field.state.value"
                    type="text"
                    placeholder="admin"
                    @blur="field.handleBlur"
                    @update:model-value="(value) => field.handleChange(String(value))"
                  />
                  <FieldError v-if="isInvalid(field)" :errors="field.state.meta.errors" />
                </Field>
              </template>
            </form.Field>
          </div>
        </div>

        <div v-if="store.testStages.length > 0 || store.isTesting" class="pt-4 border-t">
          <h4 class="text-sm font-medium mb-2">Connection Test</h4>
          <TestStagesDisplay
            :stages="store.testStages"
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
          @click="form.handleSubmit({ action: 'test' })"
        >
          Test Connection
        </Button>
        <Button
          type="button"
          :disabled="!everythingFilled || store.isTesting"
          class="hover:bg-primary/10 hover:text-primary hover:border-primary/50 transition-colors"
          @click="form.handleSubmit({ action: 'save' })"
        >
          {{ mode === 'create' ? 'Create' : 'Save' }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
