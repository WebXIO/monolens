<script setup lang="ts">
import { SidebarProps, SidebarContent, SidebarGroup, Sidebar } from './ui/sidebar';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { Button } from "@/components/ui/button"
import { ChevronDown, Database, RefreshCcw } from 'lucide-vue-next';
import { invoke } from '@tauri-apps/api/core';
import { ref } from 'vue';
import {ScrollArea} from '@/components/ui/scroll-area';

const props = defineProps<SidebarProps>()

const databases = ref<any[]>([]);

const getDatabases = async () => {
   console.log('Invoke command')
   databases.value = await invoke('get_databases')

   console.log(databases.value)
}

getDatabases()
</script>

<template>
   <Sidebar class="top-(--header-height) h-[calc(100svh-var(--header-height))]!" v-bind="props">
      <SidebarContent>
         <SidebarGroup class="flex flex-row item-center items-center-safe justify-between border-b border-border">
            <span className="text-xs font-medium uppercase tracking-wider text-muted-foreground">
               Explorer
            </span>
            <div class="flex gap-0.5">
               <TooltipProvider>
                  <Tooltip>
                     <TooltipTrigger as-child>
                        <Button variant="ghost" size="icon" class="h-6 w-6">
                           <RefreshCcw class="h-3.5 w-3.5"></RefreshCcw>
                        </Button>
                     </TooltipTrigger>
                      <TooltipContent side="bottom" class="text-xs">
                  Refresh
                </TooltipContent>
                  </Tooltip>
               </TooltipProvider>
            </div>
         </SidebarGroup>
         <SidebarGroup>
            <ScrollArea class="flex-1">
               <div v-for="(database, idx) in databases" :key="idx">
                  <div class="group flex cursor-pointer items-center gap-1 rounded-md px-2 py-1.5 text-sm transition-colors hover:bg-secondary">
                     <Button variant="ghost" size="icon" class="h-4 w-4 hover:bg-transparent">
                        <ChevronDown></ChevronDown>
                     </Button>
                     <Database></Database>
                     <span>{{ database.name }}</span>
                  </div>
               </div>
            </ScrollArea>
         </SidebarGroup>
      </SidebarContent>
   </Sidebar>
</template>