<script setup lang="ts">
import { Toaster } from 'vue-sonner'
import 'vue-sonner/style.css'
import { onMounted } from 'vue';
import Header from '@/components/Header.vue';
import AppSidebar from '@/components/AppSidebar.vue';
import { SidebarProvider, SidebarInset } from '@/components/ui/sidebar';
import ConnectionGrid from '@/components/connection/ConnectionGrid.vue';
import { useConnectionStore } from '@/stores/connectionStore';
import TabsContainer from './components/tabs/TabsContainer.vue';

const store = useConnectionStore();

onMounted(() => {
  store.loadConnections();
});
</script>

<template>
  <Toaster rich-colors theme="dark" position="top-center" />
  <div class="[--header-height:calc(--spacing(14))]">
    <SidebarProvider class="flex flex-col">
      <Header />
      <div class="flex flex-1 min-h-0">
        <template v-if="!store.activeConnection">
          <div class="flex-1">
            <ConnectionGrid />
          </div>
        </template>
        
        <template v-else>
          <AppSidebar />
          <SidebarInset>
            <TabsContainer />
          </SidebarInset>
        </template>
      </div>
    </SidebarProvider>
  </div>
</template>