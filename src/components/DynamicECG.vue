<template>
  <div class="ecg-container">
    <div class="ecg-controls">
      <button @click="togglePlay" class="control-btn">
        {{ isPlaying ? '⏸暂停' : '▶播放' }}
      </button>
      <input type="range" v-model="speed" min="0.5" max="5" step="0.1" />
      <span>速度 x{{ speed }}</span>
      <button @click="resetView" class="control-btn">重置</button>
    </div>
    <canvas ref="canvasRef" class="ecg-canvas" @wheel="handleWheel"></canvas>
    <div class="ecg-stats" v-if="currentSample">
      当前提交: 改动 {{ currentSample.total_churn }} 行, 涉及 {{ currentSample.file_count }} 个文件
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';

interface CommitSample {
  timestamp: string;
  added_lines: number;
  deleted_lines: number;
  file_count: number;
  total_churn: number;
}

const props = defineProps<{
  samples: CommitSample[];
}>();

const canvasRef = ref<HTMLCanvasElement | null>(null);
const isPlaying = ref(true);
const speed = ref(1.0);
let animationId: number | null = null;
let currentIndex = 0;
let offsetX = 0;
const pointsPerCommit = 30;
let canvasWidth = 0;
let canvasHeight = 0;

const currentSample = computed(() => props.samples[currentIndex]);

const maxChurn = computed(() => {
  if (!props.samples.length) return 1;
  return Math.max(...props.samples.map(s => s.total_churn), 1);
});

function getHealthScore(sample: CommitSample): number {
  let raw = 1 - (sample.total_churn / maxChurn.value);
  return Math.min(0.95, Math.max(0.05, raw));
}

function drawWave() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;
  ctx.clearRect(0, 0, canvasWidth, canvasHeight);

  if (props.samples.length === 0) return;

  ctx.strokeStyle = '#e2e8f0';
  ctx.lineWidth = 0.5;
  for (let y = 0; y <= 4; y++) {
    const yPos = canvasHeight - (y / 4) * canvasHeight;
    ctx.beginPath();
    ctx.moveTo(0, yPos);
    ctx.lineTo(canvasWidth, yPos);
    ctx.stroke();
  }

  const startIdx = Math.max(0, Math.floor(offsetX / pointsPerCommit));
  const endIdx = Math.min(props.samples.length - 1, Math.floor((offsetX + canvasWidth) / pointsPerCommit) + 1);

  ctx.beginPath();
  ctx.strokeStyle = '#e74c3c';
  ctx.lineWidth = 2.5;
  let firstPoint = true;

  for (let i = startIdx; i <= endIdx; i++) {
    const sample = props.samples[i];
    const health = getHealthScore(sample);
    const y = canvasHeight * (1 - health);
    const x = i * pointsPerCommit - offsetX;
    if (x < 0 || x > canvasWidth) continue;

    if (firstPoint) {
      ctx.moveTo(x, y);
      firstPoint = false;
    } else {
      const prevSample = props.samples[i-1];
      const prevHealth = getHealthScore(prevSample);
      const prevY = canvasHeight * (1 - prevHealth);
      const prevX = (i-1) * pointsPerCommit - offsetX;
      ctx.lineTo(x, y);
      if (sample.total_churn > maxChurn.value * 0.7 || sample.file_count > 3) {
        const spikeX = (prevX + x) / 2;
        const spikeY = y - 8;
        ctx.lineTo(spikeX, spikeY);
        ctx.lineTo(x, y);
      }
    }
  }
  ctx.stroke();

  const currentX = currentIndex * pointsPerCommit - offsetX;
  if (currentX >= 0 && currentX <= canvasWidth) {
    ctx.beginPath();
    ctx.moveTo(currentX, 0);
    ctx.lineTo(currentX, canvasHeight);
    ctx.strokeStyle = '#3b82f6';
    ctx.lineWidth = 1.5;
    ctx.stroke();
  }
}

function animate() {
  if (!isPlaying.value) return;
  if (currentIndex < props.samples.length - 1) {
    currentIndex++;
    const rightEdge = currentIndex * pointsPerCommit - offsetX;
    if (rightEdge > canvasWidth * 0.7) {
      offsetX = currentIndex * pointsPerCommit - canvasWidth * 0.3;
    }
    drawWave();
    playBeep(props.samples[currentIndex]);
  } else {
    isPlaying.value = false;
  }
  animationId = requestAnimationFrame(() => {
    setTimeout(animate, 1000 / (15 * speed.value));
  });
}

let audioCtx: AudioContext | null = null;
function playBeep(sample: CommitSample) {
  if (!audioCtx) {
    audioCtx = new (window.AudioContext || (window as any).webkitAudioContext)();
  }
  if (audioCtx.state === 'suspended') {
    audioCtx.resume();
  }
  const intensity = sample.total_churn / maxChurn.value;
  const frequency = 440 + intensity * 440;
  const duration = 0.1;
  const oscillator = audioCtx.createOscillator();
  const gain = audioCtx.createGain();
  oscillator.connect(gain);
  gain.connect(audioCtx.destination);
  oscillator.frequency.value = frequency;
  gain.gain.value = 0.2;
  oscillator.type = 'sine';
  oscillator.start();
  gain.gain.exponentialRampToValueAtTime(0.00001, audioCtx.currentTime + duration);
  oscillator.stop(audioCtx.currentTime + duration);
}

function togglePlay() {
  isPlaying.value = !isPlaying.value;
  if (isPlaying.value) {
    animate();
  }
}

function resetView() {
  currentIndex = 0;
  offsetX = 0;
  drawWave();
  if (!isPlaying.value) {
    isPlaying.value = true;
    animate();
  }
}

function handleWheel(e: WheelEvent) {
  offsetX = Math.min(Math.max(0, offsetX + e.deltaY * 0.5), (props.samples.length * pointsPerCommit) - canvasWidth);
  drawWave();
}

onMounted(() => {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const resize = () => {
    const container = canvas.parentElement;
    if (container) {
      canvasWidth = container.clientWidth;
      canvasHeight = 300;
      canvas.width = canvasWidth;
      canvas.height = canvasHeight;
      drawWave();
    }
  };
  window.addEventListener('resize', resize);
  resize();
  animate();
});

onUnmounted(() => {
  if (animationId) cancelAnimationFrame(animationId);
  if (audioCtx) audioCtx.close();
});
</script>

<style scoped>
.ecg-container {
  background: white;
  border-radius: 12px;
  padding: 16px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.1);
  margin-bottom: 20px;
}
.ecg-controls {
  display: flex;
  gap: 16px;
  align-items: center;
  margin-bottom: 12px;
}
.control-btn {
  background: #3b82f6;
  border: none;
  color: white;
  padding: 6px 12px;
  border-radius: 20px;
  cursor: pointer;
}
.ecg-canvas {
  width: 100%;
  height: 300px;
  background: #f8fafc;
  border-radius: 8px;
  cursor: ew-resize;
}
.ecg-stats {
  margin-top: 12px;
  font-size: 0.9rem;
  color: #475569;
}
</style>