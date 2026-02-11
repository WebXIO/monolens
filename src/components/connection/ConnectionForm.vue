<script setup lang="ts">
import { toTypedSchema } from "@vee-validate/zod";
import * as z from "zod";

import {
   FormControl,
   FormField,
   FormItem,
   FormLabel,
   FormMessage,
} from '@/components/ui/form'
import { AuthenticationKind, ConnectionType, ConnectorKind } from "@/domains/connections";
import { useForm } from "vee-validate";
import Input from "../ui/input/Input.vue";
import Button from "../ui/button/Button.vue";
import { invoke } from "@tauri-apps/api/core";

const formSchema = toTypedSchema(z.object({
   uri: z.string(),
   port: z.number().default(27017),
   name: z.string(),
   authentication: z.object({
      kind: z.string().default(AuthenticationKind.NONE),
      username: z.string().optional(),
      password: z.string().optional(),
      database: z.string().optional(),
   }),
   connectionType: z.string().default(ConnectionType.Standalone),
   connector: z.string().default(ConnectorKind.MongoDb),
}))

const form = useForm({
   validationSchema: formSchema
})

const onSubmit = form.handleSubmit(async (values) => {
   const res = await invoke("test_connection", {connection: {...values, id: ""}});

   console.log(res);
})
</script>

<template>
   <form @submit="onSubmit">
      <FormField v-slot="{ componentField }" name="name">
         <FormItem>
            <FormLabel>Name</FormLabel>
            <FormControl>
               <Input type="text" placeholder="Name of the connection" v-bind="componentField" />
            </FormControl>
            <FormMessage />
         </FormItem>
      </FormField>
      <FormField v-slot="{ componentField }" name="uri">
         <FormItem>
            <FormLabel>Uri</FormLabel>
            <FormControl>
               <Input type="text" placeholder="Database-Uri" v-bind="componentField" />
            </FormControl>
            <FormMessage />
         </FormItem>
      </FormField>
      <FormField v-slot="{ componentField }" name="port">
         <FormItem>
            <FormLabel>Port</FormLabel>
            <FormControl>
               <Input type="number" placeholder="27017" v-bind="componentField" />
            </FormControl>
            <FormMessage />
         </FormItem>
      </FormField>
      <FormField v-slot="{ componentField }" name="authentication.username">
         <FormItem>
            <FormLabel>Username</FormLabel>
            <FormControl>
               <Input type="text" placeholder="Username" v-bind="componentField" />
            </FormControl>
            <FormMessage />
         </FormItem>
      </FormField>
      <FormField v-slot="{ componentField }" name="authentication.password">
         <FormItem>
            <FormLabel>Password</FormLabel>
            <FormControl>
               <Input type="password" placeholder="Password" v-bind="componentField" />
            </FormControl>
            <FormMessage />
         </FormItem>
      </FormField>
      <FormField v-slot="{ componentField }" name="authentication.database">
         <FormItem>
            <FormLabel>Database</FormLabel>
            <FormControl>
               <Input type="text" placeholder="Database-Name" v-bind="componentField" />
            </FormControl>
            <FormMessage />
         </FormItem>
      </FormField>
      <Button type="submit">Submit</Button>
   </form>
</template>