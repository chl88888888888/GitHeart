<script setup lang="ts">
import { ref } from 'vue';

const emit = defineEmits<{
  (e: 'analyze', url: string, token?: string): void;
}>();

const repoUrl = ref('');
const githubToken = ref('');
const showTokenInput = ref(false);
const isValid = ref(true);
const errorMsg = ref('');

function validateAndSubmit() {
  const url = repoUrl.value.trim();
  if (!url) {
    isValid.value = false;
    errorMsg.value = '请输入 GitHub 仓库地址';
    return;
  }

  const patterns = [
    /^https?:\/\/github\.com\/[\w.-]+\/[\w.-]+(\.git)?$/,
    /^github\.com\/[\w.-]+\/[\w.-]+(\.git)?$/,
    /^[\w.-]+\/[\w.-]+$/,
  ];
  const isValidFormat = patterns.some(p => p.test(url));
  if (!isValidFormat) {
    isValid.value = false;
    errorMsg.value = '无效的 GitHub 仓库地址，支持格式: owner/repo 或 https://github.com/owner/repo';
    return;
  }

  isValid.value = true;
  errorMsg.value = '';
  emit('analyze', url, githubToken.value.trim() || undefined);
}

function toggleTokenInput() {
  showTokenInput.value = !showTokenInput.value;
}
</script>

<template>
  <div class="github-input">
    <div class="input-group">
      <input
        v-model="repoUrl"
        type="text"
        placeholder="输入 GitHub 仓库，如: facebook/react"
        @keyup.enter="validateAndSubmit"
        :class="{ 'error': !isValid }"
      />
      <button @click="validateAndSubmit" class="analyze-btn">
        分析云端仓库
      </button>
    </div>

    <div class="token-option">
      <button type="button" @click="toggleTokenInput" class="toggle-token-btn">
        🔑 {{ showTokenInput ? '隐藏' : '添加 GitHub Token（提升限额）' }}
      </button>
      <input
        v-if="showTokenInput"
        v-model="githubToken"
        type="password"
        placeholder="GitHub Personal Access Token (可选)"
        class="token-input"
      />
    </div>

    <p v-if="!isValid" class="error-msg">{{ errorMsg }}</p>
  </div>
</template>

<style scoped>
.github-input {
  width: 100%;
  max-width: 500px;
}

.input-group {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
}

.input-group input {
  flex: 1;
  padding: 10px 16px;
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  font-size: 1rem;
  background: white;
  transition: border-color 0.2s;
}

.input-group input:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.input-group input.error {
  border-color: #dc2626;
}

.analyze-btn {
  background: #3b82f6;
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 8px;
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  transition: background 0.2s;
}

.analyze-btn:hover {
  background: #2563eb;
}

.token-option {
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.toggle-token-btn {
  background: none;
  border: none;
  color: #64748b;
  font-size: 0.8rem;
  cursor: pointer;
  text-align: left;
  padding: 4px 0;
}

.toggle-token-btn:hover {
  color: #3b82f6;
  text-decoration: underline;
}

.token-input {
  padding: 8px 12px;
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  font-size: 0.85rem;
  font-family: monospace;
  background: #f8fafc;
}

.error-msg {
  color: #dc2626;
  font-size: 0.85rem;
  margin-top: 6px;
  margin-left: 4px;
}
</style>