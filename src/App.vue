<script setup lang="ts">
import Header from '@/components/Header.vue';
import AppSidebar from '@/components/AppSidebar.vue';
import { SidebarProvider, SidebarInset } from '@/components/ui/sidebar';
import ConnectionForm from './components/connection/ConnectionForm.vue';
</script>

<template>

  <div class="[--header-height:calc(--spacing(14))]">
    <SidebarProvider class="flex flex-col">
      <Header />
       <div class="flex flex-1">
         <AppSidebar />
         <SidebarInset>
          <div class="flex flex-1 flex-col gap-4 p-4">
            <ConnectionForm />
          </div>
        </SidebarInset>
       </div>
    </SidebarProvider>
  </div>
</template>