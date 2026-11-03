<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useForm } from '@tanstack/vue-form';

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Field, FieldDescription, FieldError, FieldLabel } from '@/components/ui/field';
import Input from '@/components/ui/input/Input.vue';
import Button from '@/components/ui/button/Button.vue';
import TestStagesDisplay from './TestStagesDisplay.vue';
import { ChevronDown, ChevronRight, Eye, EyeOff } from '@lucide/vue';

import { useConnectionStore } from '@/stores/connectionStore';
import { AuthenticationKind, type Connection } from '@/domains/connections';
import { useLogger } from '@/composables/useLogger';
import {
  connectionFormSchema,
  connectionFromFormValues,
  defaultFormValues,
  formValuesFromConnection,
  type ConnectionFormValues,
  type ConnectionMode,
} from './connectionForm';

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

// bound, not static: vue-tsc 3 misreads a // inside a static attribute value
const URI_PLACEHOLDER = 'mongodb+srv://cluster0.example.net/?retryWrites=true';

type SubmitAction = { action: 'test' | 'save' };

const form = useForm({
  defaultValues: defaultFormValues(),
  validators: {
    onSubmit: connectionFormSchema,
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
const showPassword = ref(false);
const showOptions = ref(false);

function isInvalid(field: { state: { meta: { isTouched: boolean; isValid: boolean } } }) {
  return field.state.meta.isTouched && !field.state.meta.isValid;
}

// filled on every opening: the same connection opened twice doesn't change props.connection
watch(() => [props.open, props.connection] as const, async ([open, conn]) => {
  if (open && conn) {
    let password = '';
    if (conn.authentication.kind !== AuthenticationKind.NONE && props.mode === 'edit') {
      password = (await store.getConnectionPassword(conn.id)) ?? '';
    }
    const formValues = formValuesFromConnection(conn, password);
    form.reset(formValues);
    showOptions.value = hasOptions(formValues);
  }
}, { immediate: true });

watch(isOpen, (open) => {
  if (!open) {
    form.reset(defaultFormValues());
    store.clearTestState();
    showPassword.value = false;
    showOptions.value = false;
  }
});

function hasOptions(v: ConnectionFormValues) {
  return v.tls || Boolean(v.replicaSet.trim()) || Boolean(v.timeoutSecs.trim());
}

function setMode(mode: ConnectionMode) {
  form.setFieldValue('mode', mode);
}

async function handleTest(formValues: ConnectionFormValues) {
  logger.debug('testing connection');
  await store.testConnection(connectionFromFormValues(formValues));
}

async function handleSave(formValues: ConnectionFormValues) {
  const connectionData = connectionFromFormValues(formValues);

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

const isHostMode = computed(() => values.value.mode === 'host');
const useAuth = computed(() => values.value.useAuth);
const useTls = computed(() => values.value.tls);
</script>

<template>
  <Dialog v-model:open="isOpen">
    <DialogContent class="sm:max-w-[700px] max-h-[90vh] overflow-y-auto">
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

        <div class="inline-flex rounded-md border border-border p-0.5" role="radiogroup" aria-label="Connect by">
          <Button
            :variant="isHostMode ? 'secondary' : 'ghost'"
            :aria-checked="isHostMode"
            role="radio"
            size="sm"
            type="button"
            @click="setMode('host')"
          >
            Host and port
          </Button>
          <Button
            :variant="!isHostMode ? 'secondary' : 'ghost'"
            :aria-checked="!isHostMode"
            role="radio"
            size="sm"
            type="button"
            @click="setMode('uri')"
          >
            Connection string
          </Button>
        </div>

        <div v-if="isHostMode" class="grid grid-cols-3 gap-4">
          <form.Field name="host">
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

        <form.Field v-else name="uri">
          <template #default="{ field }">
            <Field :data-invalid="isInvalid(field)">
              <FieldLabel :for="field.name">Connection String</FieldLabel>
              <Input
                :id="field.name"
                :name="field.name"
                :model-value="field.state.value"
                :aria-invalid="isInvalid(field)"
                type="text"
                :placeholder="URI_PLACEHOLDER"
                autocomplete="off"
                spellcheck="false"
                @blur="field.handleBlur"
                @update:model-value="(value) => field.handleChange(String(value))"
              />
              <FieldDescription>
                The string carries all options. A username and password in it are moved out on save: the password goes to the system keyring.
              </FieldDescription>
              <FieldError v-if="isInvalid(field)" :errors="field.state.meta.errors" />
            </Field>
          </template>
        </form.Field>

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
                    :aria-invalid="isInvalid(field)"
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
                      :aria-invalid="isInvalid(field)"
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
                      :aria-label="showPassword ? 'Hide password' : 'Show password'"
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
                <Field>
                  <FieldLabel :for="field.name">Auth Database</FieldLabel>
                  <Input
                    :id="field.name"
                    :name="field.name"
                    :model-value="field.state.value"
                    type="text"
                    :placeholder="isHostMode ? 'admin' : 'from the string, else admin'"
                    @blur="field.handleBlur"
                    @update:model-value="(value) => field.handleChange(String(value))"
                  />
                </Field>
              </template>
            </form.Field>
          </div>
        </div>

        <div v-if="isHostMode" class="space-y-3">
          <button
            type="button"
            class="flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground transition-colors"
            :aria-expanded="showOptions"
            @click="showOptions = !showOptions"
          >
            <ChevronDown v-if="showOptions" class="h-4 w-4" />
            <ChevronRight v-else class="h-4 w-4" />
            Options
          </button>

          <div v-if="showOptions" class="space-y-4 pl-4 border-l-2 border-muted">
            <form.Field name="tls">
              <template #default="{ field }">
                <Field orientation="horizontal">
                  <input
                    :id="field.name"
                    :checked="field.state.value"
                    type="checkbox"
                    class="h-4 w-4 rounded border-input"
                    @change="(e: Event) => field.handleChange((e.target as HTMLInputElement).checked)"
                  />
                  <FieldLabel :for="field.name">Use TLS</FieldLabel>
                </Field>
              </template>
            </form.Field>

            <div v-if="useTls" class="grid grid-cols-3 items-end gap-4">
              <form.Field name="tlsCaFile">
                <template #default="{ field }">
                  <Field class="col-span-2">
                    <FieldLabel :for="field.name">CA File</FieldLabel>
                    <Input
                      :id="field.name"
                      :name="field.name"
                      :model-value="field.state.value"
                      type="text"
                      placeholder="/path/to/ca.pem (empty: system certificates)"
                      @blur="field.handleBlur"
                      @update:model-value="(value) => field.handleChange(String(value))"
                    />
                  </Field>
                </template>
              </form.Field>

              <form.Field name="tlsAllowInvalidCertificates">
                <template #default="{ field }">
                  <Field orientation="horizontal" class="h-9">
                    <input
                      :id="field.name"
                      :checked="field.state.value"
                      type="checkbox"
                      class="h-4 w-4 rounded border-input"
                      @change="(e: Event) => field.handleChange((e.target as HTMLInputElement).checked)"
                    />
                    <FieldLabel :for="field.name">Allow invalid certificates</FieldLabel>
                  </Field>
                </template>
              </form.Field>
            </div>

            <div class="grid grid-cols-3 gap-4">
              <form.Field name="replicaSet">
                <template #default="{ field }">
                  <Field class="col-span-2">
                    <FieldLabel :for="field.name">Replica Set</FieldLabel>
                    <Input
                      :id="field.name"
                      :name="field.name"
                      :model-value="field.state.value"
                      type="text"
                      placeholder="rs0 (empty: none)"
                      @blur="field.handleBlur"
                      @update:model-value="(value) => field.handleChange(String(value))"
                    />
                  </Field>
                </template>
              </form.Field>

              <form.Field name="timeoutSecs">
                <template #default="{ field }">
                  <Field :data-invalid="isInvalid(field)">
                    <FieldLabel :for="field.name">Timeout (s)</FieldLabel>
                    <Input
                      :id="field.name"
                      :name="field.name"
                      :model-value="field.state.value"
                      :aria-invalid="isInvalid(field)"
                      type="text"
                      inputmode="numeric"
                      placeholder="30"
                      @blur="field.handleBlur"
                      @update:model-value="(value) => field.handleChange(String(value))"
                    />
                    <FieldError v-if="isInvalid(field)" :errors="field.state.meta.errors" />
                  </Field>
                </template>
              </form.Field>
            </div>
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
          class="hover:bg-info/10 hover:text-info hover:border-info/50 transition-colors"
          @click="form.handleSubmit({ action: 'test' })"
        >
          Test Connection
        </Button>
        <Button
          type="button"
          :disabled="store.isTesting"
          class="hover:bg-primary/10 hover:text-primary hover:border-primary/50 transition-colors"
          @click="form.handleSubmit({ action: 'save' })"
        >
          {{ mode === 'create' ? 'Create' : 'Save' }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
