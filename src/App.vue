<script setup lang="ts">
import { onMounted, computed } from 'vue';
import Header from '@/components/Header.vue';
import AppSidebar from '@/components/AppSidebar.vue';
import { SidebarProvider, SidebarInset } from '@/components/ui/sidebar';
import ConnectionGrid from '@/components/connection/ConnectionGrid.vue';
import { useConnectionStore } from '@/stores/connectionStore';
import TabsContainer from './components/tabs/TabsContainer.vue';

const store = useConnectionStore();

const activeConnection = computed(() => store.activeConnection.value);

onMounted(() => {
  store.loadConnections();
});
</script>

<template>
  <div class="[--header-height:calc(--spacing(14))]">
    <SidebarProvider class="flex flex-col">
      <Header />
      <div class="flex flex-1">
        <template v-if="!activeConnection">
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