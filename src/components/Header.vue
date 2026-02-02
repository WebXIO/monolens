<script setup lang="ts">
import Logo from './Logo.vue';
import DropdownMenu from '@/components/ui/dropdown-menu/DropdownMenu.vue';
import DropdownMenuTrigger from '@/components/ui/dropdown-menu/DropdownMenuTrigger.vue';
import Button from '@/components/ui/button/Button.vue';
import { ChevronDown } from 'lucide-vue-next';
import DropdownMenuContent from '@/components/ui/dropdown-menu/DropdownMenuContent.vue';
import DropdownMenuLabel from '@/components/ui/dropdown-menu/DropdownMenuLabel.vue';
import DropdownMenuItem from '@/components/ui/dropdown-menu/DropdownMenuItem.vue';
import DropdownMenuSeparator from '@/components/ui/dropdown-menu/DropdownMenuSeparator.vue';
</script>

<template>
   <header class="flex h-(--header-height) items-center justify-between border-b border-border bg-sidebar px-3">
      <div class="flex items-center gap-4">

         <Logo />

         <div class="h-5 w-px bg-border"></div>

         <DropdownMenu>
            <DropdownMenuTrigger as-child>
               <Button variant="ghost" class="h-8 gap-2 px-3 text-sm">G</Button>
               <ChevronDown class="h-3.5 w-3.5 text-muted-foreground" />
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start" className="w-64">
               <DropdownMenuLabel className="text-xs">Connections</DropdownMenuLabel>

               <DropdownMenuItem key="test" :text-value="'test'">

               </DropdownMenuItem>

               <DropdownMenuSeparator />
               <DropdownMenuItem>
                  <span className="text-primary">+ New Connection</span>
               </DropdownMenuItem>
            </DropdownMenuContent>
         </DropdownMenu>
      </div>
   </header>
</template>