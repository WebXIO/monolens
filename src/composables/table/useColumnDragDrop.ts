import { ref, type Ref } from 'vue';
import type { ColumnOrderState, Table } from '@tanstack/vue-table';

export function useColumnDragDrop(table: Table<any>, columnOrder: Ref<ColumnOrderState>) {
   const draggedColumnId = ref<string | null>(null);
   const dropTargetColumnId = ref<string | null>(null);
   const didDrag = ref(false);

   function onDragStart(event: DragEvent, columnId: string) {
      draggedColumnId.value = columnId;
      didDrag.value = false;
      if (event.dataTransfer) {
         event.dataTransfer.effectAllowed = 'move';
         event.dataTransfer.setData('text/plain', columnId);

         const ghost = document.createElement('div');
         ghost.textContent = columnId;
         ghost.style.cssText =
            'position:fixed;left:-9999px;top:-9999px;padding:4px 10px;border-radius:4px;font-size:12px;font-weight:500;background:var(--secondary);color:var(--foreground);border:1px solid var(--border);white-space:nowrap;';
         document.body.appendChild(ghost);
         event.dataTransfer.setDragImage(ghost, ghost.offsetWidth / 2, ghost.offsetHeight / 2);
         requestAnimationFrame(() => document.body.removeChild(ghost));
      }
   }

   function onDragOver(event: DragEvent, columnId: string) {
      event.preventDefault();
      if (draggedColumnId.value && draggedColumnId.value !== columnId) {
         dropTargetColumnId.value = columnId;
      }
   }

   function onDragLeave() {
      dropTargetColumnId.value = null;
   }

   function onDrop(event: DragEvent, targetColumnId: string) {
      event.preventDefault();
      dropTargetColumnId.value = null;

      const sourceId = draggedColumnId.value;
      if (!sourceId || sourceId === targetColumnId) return;

      didDrag.value = true;
      const currentOrder = [...columnOrder.value];
      const sourceIndex = currentOrder.indexOf(sourceId);
      const targetIndex = currentOrder.indexOf(targetColumnId);
      if (sourceIndex === -1 || targetIndex === -1) return;

      currentOrder.splice(sourceIndex, 1);
      currentOrder.splice(targetIndex, 0, sourceId);

      table.setColumnOrder(currentOrder);
   }

   function onDragEnd() {
      draggedColumnId.value = null;
      dropTargetColumnId.value = null;
   }

   function onHeaderClick(event: MouseEvent, handler?: (e: MouseEvent) => void) {
      if (didDrag.value) {
         didDrag.value = false;
         return;
      }
      handler?.(event);
   }

   return {
      draggedColumnId,
      dropTargetColumnId,
      onDragStart,
      onDragOver,
      onDragLeave,
      onDrop,
      onDragEnd,
      onHeaderClick,
   };
}
