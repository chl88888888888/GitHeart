<script setup lang="ts">
import { computed } from 'vue';
import type { FileStats } from '../composables/useGitAnalyzer';

const props = defineProps<{
  files: FileStats[];
}>();

const hotspots = computed(() => props.files.slice(0, 15));

function getRiskLevel(file: FileStats): { text: string; class: string } {
  if (file.total_changes > 100 || file.churn_index > 150)
    return { text: '🔴 高危', class: 'high' };
  if (file.total_changes > 50 || file.churn_index > 80)
    return { text: '🟡 中危', class: 'medium' };
  return { text: '🟢 低危', class: 'low' };
}
</script>

<template>
  <div class="hotspot-section">
    <h2>🔥 代码修改热点 Top 15</h2>
    <table>
      <thead>
        <tr>
          <th>文件路径</th>
          <th>改动次数</th>
          <th>+ 行</th>
          <th>- 行</th>
          <th>流失指数</th>
          <th>风险评级</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="file in hotspots" :key="file.path">
          <td class="file-path" :title="file.path">{{ file.path }}</td>
          <td class="changes">{{ file.total_changes }}</td>
          <td class="added">{{ file.lines_added }}</td>
          <td class="deleted">{{ file.lines_deleted }}</td>
          <td class="churn">{{ file.churn_index.toFixed(1) }}</td>
          <td>
            <span :class="['risk-badge', getRiskLevel(file).class]">
              {{ getRiskLevel(file).text }}
            </span>
          </td>
        </tr>
      </tbody>
    </table>
    <div v-if="files.length === 0" class="no-data">
      暂无文件修改数据
    </div>
    <div class="table-footnote">
      *流失指数 = (新增行 + 删除行) / 修改次数，数值越高代表单次改动越剧烈。
    </div>
  </div>
</template>

<style scoped>
.hotspot-section {
  background: white;
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  padding: 20px;
}

h2 {
  margin-bottom: 20px;
  font-size: 1.4rem;
  font-weight: 500;
  color: #1e293b;
}

table {
  width: 100%;
  border-collapse: collapse;
}

th,
td {
  padding: 12px 16px;
  text-align: left;
  border-bottom: 1px solid #e2e8f0;
}

th {
  background: #f1f5f9;
  font-weight: 600;
  color: #334155;
}

tr:hover {
  background: #f8fafc;
}

.file-path {
  max-width: 400px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: 'Menlo', 'Consolas', monospace;
  font-size: 0.9rem;
}

.changes,
.added,
.deleted,
.churn {
  text-align: center;
  width: 80px;
}

.changes {
  font-weight: 600;
}

.added {
  color: #15803d;
}

.deleted {
  color: #b91c1c;
}

.churn {
  font-weight: 500;
}

.risk-badge {
  padding: 4px 10px;
  border-radius: 20px;
  font-size: 0.85rem;
  font-weight: 500;
}

.risk-badge.high {
  background: #fee2e2;
  color: #b91c1c;
}

.risk-badge.medium {
  background: #fef9c3;
  color: #a16207;
}

.risk-badge.low {
  background: #dcfce7;
  color: #15803d;
}

.no-data {
  text-align: center;
  padding: 40px;
  color: #94a3b8;
}

.table-footnote {
  margin-top: 12px;
  font-size: 0.8rem;
  color: #64748b;
  text-align: right;
}
</style>