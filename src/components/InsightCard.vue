<script setup lang="ts">
defineProps<{
  insights: string[];
}>();
</script>

<template>
  <div v-if="insights.length" class="insight-card">
    <h3>智能洞察</h3>
    <ul>
      <li v-for="(insight, idx) in insights" :key="idx">{{ insight }}</li>
    </ul>
  </div>
</template>

<style scoped>
.insight-card {
  background: #fffbeb;
  border-left: 4px solid #f59e0b;
  border-radius: 8px;
  padding: 16px 20px;
  margin-bottom: 24px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

.insight-card h3 {
  margin: 0 0 12px 0;
  font-size: 1.1rem;
  color: #b45309;
}

.insight-card ul {
  margin: 0;
  padding-left: 20px;
}

.insight-card li {
  margin-bottom: 8px;
  color: #78350f;
  line-height: 1.5;
}
</style>