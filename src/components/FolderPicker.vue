<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog';

const emit = defineEmits<{
  (e: 'select', path: string): void;
}>();

async function selectFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
  });
  if (selected && typeof selected === 'string') {
    emit('select', selected);
  }
}
</script>

<template>
  <button @click="selectFolder" class="picker-btn">
    选择本地 Git 仓库
  </button>
</template>

<style scoped>
.picker-btn {
  background: white;
  color: #1e293b;
  border: 1px solid #cbd5e1;
  padding: 10px 20px;
  border-radius: 8px;
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
}
.picker-btn:hover {
  background: #f8fafc;
  border-color: #94a3b8;
}
</style>