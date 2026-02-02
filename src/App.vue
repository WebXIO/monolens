<script setup lang="ts">
import Header from '@/components/Header.vue';
import AppSidebar from '@/components/AppSidebar.vue';
import { SidebarProvider, SidebarInset } from '@/components/ui/sidebar';
</script>

<template>

  <div class="[--header-height:calc(--spacing(14))]">
    <SidebarProvider class="flex flex-col">
      <Header />
       <div class="flex flex-1">
         <AppSidebar />
         <SidebarInset>
          <div class="flex flex-1 flex-col gap-4 p-4">
            <div class="grid auto-rows-min gap-4 md:grid-cols-3">
              <div class="bg-muted/50 aspect-video rounded-xl" />
              <div class="bg-muted/50 aspect-video rounded-xl" />
              <div class="bg-muted/50 aspect-video rounded-xl" />
            </div>
            <div class="bg-muted/50 min-h-[100vh] flex-1 rounded-xl md:min-h-min" />
          </div>
        </SidebarInset>
       </div>
    </SidebarProvider>
  </div>
</template>