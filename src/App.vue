<script setup lang="ts">
import { ref } from 'vue';
import FolderPicker from './components/FolderPicker.vue';
import GithubInput from './components/GithubInput.vue';
import ECGChart from './components/ECGChart.vue';
import HotspotTable from './components/HotspotTable.vue';
import InsightCard from './components/InsightCard.vue';
import { useGitAnalyzer } from './composables/useGitAnalyzer';

const { loading, result, error, analyzeLocal, analyzeGitHub } = useGitAnalyzer();

const currentSource = ref<'local' | 'github' | null>(null);

function handleLocalSelect(path: string) {
  currentSource.value = 'local';
  analyzeLocal(path);
}

function handleGithubAnalyze(url: string, token?: string) {
  currentSource.value = 'github';
  analyzeGitHub(url, token);
}
</script>

<template>
  <div class="app">
    <header>
      <div class="title-section">
        <h1>🫀 Git 心电图</h1>
        <span class="subtitle">基于变更频率的代码健康度可视化</span>
      </div>
      <div class="actions">
        <FolderPicker @select="handleLocalSelect" />
        <GithubInput @analyze="handleGithubAnalyze" />
      </div>
    </header>

    <main>
      <div v-if="loading" class="status-box">
        <div class="spinner"></div>
        <p>⏳ 正在分析{{ currentSource === 'github' ? ' GitHub 仓库' : '' }}，请稍候...</p>
        <p v-if="currentSource === 'github'" class="hint">首次分析可能需要几秒到几十秒，取决于仓库大小</p>
      </div>

      <div v-else-if="error" class="status-box error">
        <p>❌ 分析失败</p>
        <p class="error-detail">{{ error }}</p>
      </div>

      <template v-else-if="result">
        <div class="summary-bar">
          <span class="badge">📁 {{ result.repo_path }}</span>
          <span class="badge">📝 已分析 {{ result.total_commits }} 个提交</span>
          <span v-if="currentSource === 'github'" class="badge github">☁️ GitHub 云端数据</span>
        </div>

        <InsightCard :insights="result.insights" />

        <ECGChart :data="result.monthly_churn" />
        <HotspotTable :files="result.file_stats" />
      </template>

      <div v-else class="status-box welcome">
        <div class="welcome-icon">📂</div>
        <h2>欢迎使用 Git 心电图</h2>
        <p>选择本地 Git 仓库或输入 GitHub 仓库地址开始分析</p>
        <p class="hint">GitHub 分析建议添加 Personal Access Token 以提升速率限制</p>
      </div>
    </main>
  </div>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding-bottom: 16px;
  border-bottom: 1px solid #e2e8f0;
}

.title-section h1 {
  font-size: 2rem;
  font-weight: 600;
  background: linear-gradient(135deg, #e74c3c, #c0392b);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  margin-bottom: 4px;
}

.subtitle {
  color: #64748b;
  font-size: 0.9rem;
}

.actions {
  display: flex;
  flex-direction: column;
  gap: 12px;
  align-items: flex-end;
}

.status-box {
  background: white;
  border-radius: 12px;
  padding: 60px 20px;
  text-align: center;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.status-box.error {
  background: #fef2f2;
  color: #b91c1c;
}

.error-detail {
  font-family: monospace;
  margin-top: 10px;
  font-size: 0.9rem;
}

.spinner {
  width: 40px;
  height: 40px;
  margin: 0 auto 20px;
  border: 4px solid #e2e8f0;
  border-top-color: #3b82f6;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.summary-bar {
  display: flex;
  gap: 16px;
  margin-bottom: 8px;
  flex-wrap: wrap;
}

.badge {
  background: #eef2ff;
  color: #1e40af;
  padding: 6px 14px;
  border-radius: 20px;
  font-size: 0.9rem;
  font-weight: 500;
}

.badge.github {
  background: #f0fdf4;
  color: #166534;
}

.welcome-icon {
  font-size: 64px;
  margin-bottom: 20px;
}

.welcome h2 {
  margin-bottom: 10px;
  color: #1e293b;
}

.welcome p {
  color: #64748b;
  margin-bottom: 8px;
}

.hint {
  font-size: 0.9rem;
  margin-top: 20px;
  color: #94a3b8;
}
</style>