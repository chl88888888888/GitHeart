<script setup lang="ts">
import { computed } from 'vue';
import VChart from 'vue-echarts';
import { use } from 'echarts/core';
import { LineChart } from 'echarts/charts';
import { CanvasRenderer } from 'echarts/renderers';
import { GridComponent, TooltipComponent, TitleComponent } from 'echarts/components';
import type { MonthlyChurn } from '../composables/useGitAnalyzer';

use([LineChart, CanvasRenderer, GridComponent, TooltipComponent, TitleComponent]);

const props = defineProps<{
  data: MonthlyChurn[];
}>();

const chartOption = computed(() => ({
  title: {
    text: '代码提交心电图',
    left: 'center',
    top: 5,
    textStyle: { fontSize: 16, fontWeight: 'normal' }
  },
  tooltip: {
    trigger: 'axis',
    axisPointer: { type: 'shadow' },
    formatter: (params: any) => {
      const p = params[0];
      return `${p.name}<br/>提交次数: ${p.value}`;
    }
  },
  xAxis: {
    type: 'category',
    data: props.data.map(d => d.month),
    name: '月份',
    nameLocation: 'middle',
    nameGap: 30,
    axisLabel: { rotate: props.data.length > 12 ? 45 : 0 }
  },
  yAxis: {
    type: 'value',
    name: '提交次数',
    nameLocation: 'middle',
    nameGap: 40,
    minInterval: 1
  },
  series: [{
    data: props.data.map(d => d.changes),
    type: 'line',
    smooth: true,
    lineStyle: { color: '#e74c3c', width: 3 },
    areaStyle: { color: 'rgba(231, 76, 60, 0.08)' },
    symbol: 'circle',
    symbolSize: 8,
    name: '改动频次'
  }],
  grid: {
    left: '8%',
    right: '5%',
    bottom: '12%',
    top: '18%',
    containLabel: false
  }
}));
</script>

<template>
  <div class="chart-container">
    <v-chart :option="chartOption" autoresize />
  </div>
</template>

<style scoped>
.chart-container {
  width: 100%;
  height: 400px;
  background: white;
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.1);
  padding: 16px;
  margin-bottom: 24px;
}
</style>