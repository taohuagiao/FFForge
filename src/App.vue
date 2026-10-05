<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import PresetPicker from "./components/PresetPicker.vue";
import AddFilesButton from "./components/AddFilesButton.vue";
import JobRow from "./components/JobRow.vue";
import { store, startAll, clearFinished } from "./stores/jobs";
import {
  onProgress,
  onJobLog,
  onJobError,
  onJobFinished,
} from "./lib/ipc";
import type { UnlistenFn } from "@tauri-apps/api/event";

const unlisteners: UnlistenFn[] = [];
const hasJobs = computed(() => store.jobs.length > 0);
const pendingCount = computed(
  () => store.jobs.filter((j) => j.status === "pending" || j.status === "probing").length,
);
const doneCount = computed(() => store.jobs.filter((j) => j.status === "done").length);
const failedCount = computed(() => store.jobs.filter((j) => j.status === "failed").length);
const summary = ref("");

onMounted(async () => {
  unlisteners.push(
    await onProgress((p) => store && handleProgressSafe(p)),
    await onJobLog((l) => handleLogSafe(l)),
    await onJobError((e) => handleErrorSafe(e)),
    await onJobFinished((f) => handleFinishedSafe(f)),
  );
});

onUnmounted(() => {
  unlisteners.forEach((u) => u());
});

// 事件处理经由 store 导出的函数（避免循环导入，这里动态引入）。
import {
  handleProgress,
  handleJobLog,
  handleJobError,
  handleJobFinished,
} from "./stores/jobs";

function handleProgressSafe(p: { id: number; outTimeMs: number; percent: number | null; fps: number | null; speed: number | null }) {
  handleProgress(p.id, p);
}
function handleLogSafe(l: { id: number; line: string }) {
  handleJobLog(l.id, l.line);
}
function handleErrorSafe(e: { id: number; message: string }) {
  handleJobError(e.id, e.message);
}
function handleFinishedSafe(f: { id: number; code: number; cancelled: boolean }) {
  handleJobFinished(f.id, f.code, f.cancelled);
}

async function onStartAll() {
  summary.value = "";
  try {
    await startAll();
    summary.value = "队列已处理完毕";
  } catch (e) {
    summary.value = String(e);
  }
}
</script>

<template>
  <main class="app">
    <header class="app-header">
      <div>
        <h1>FFForge</h1>
        <p class="subtitle">FFmpeg 图形界面 · 转码 / 进度 / 取消</p>
      </div>
      <div class="header-actions">
        <AddFilesButton />
      </div>
    </header>

    <PresetPicker />

    <section class="queue">
      <div class="queue-bar">
        <h2>任务队列</h2>
        <div class="queue-actions">
          <span v-if="pendingCount" class="queue-hint">{{ pendingCount }} 个待处理</span>
          <button
            type="button"
            class="btn primary"
            :disabled="!pendingCount || store.running"
            @click="onStartAll"
          >
            {{ store.running ? "转码中…" : "开始转码" }}
          </button>
          <button
            type="button"
            class="btn"
            :disabled="!hasJobs || store.running"
            @click="clearFinished"
          >
            清除已完成
          </button>
        </div>
      </div>

      <p v-if="summary" class="summary">{{ summary }}</p>

      <div v-if="hasJobs" class="job-list">
        <JobRow v-for="(job, i) in store.jobs" :key="job.input + i" :job="job" :index="i" />
      </div>
      <div v-else class="empty">
        <p>队列是空的。</p>
        <p class="muted">点击「添加文件」选择要转码的媒体文件（支持中文路径、批量多选）。</p>
      </div>

      <footer v-if="hasJobs" class="queue-footer">
        <span>完成 {{ doneCount }}</span>
        <span v-if="failedCount">失败 {{ failedCount }}</span>
        <span>共 {{ store.jobs.length }}</span>
      </footer>
    </section>
  </main>
</template>

<style>
/* ---------- 全局样式（原生 CSS，无组件库） ---------- */
:root {
  color-scheme: light dark;
  font-family: "Segoe UI", "Microsoft YaHei", system-ui, sans-serif;
  font-size: 14px;
  line-height: 1.5;
  --text: #1f2328;
  --text-dim: #6e7681;
  --border: #d0d7de;
  --surface: #ffffff;
  --surface-active: #eef4ff;
  --track: #eaeef2;
  --accent: #316dca;
  --accent-text: #ffffff;
  --ok: #1a7f37;
  --warn: #9a6700;
  --err: #cf222e;
  background-color: #f6f8fa;
  color: var(--text);
}

@media (prefers-color-scheme: dark) {
  :root {
    --text: #e6edf3;
    --text-dim: #8b949e;
    --border: #30363d;
    --surface: #161b22;
    --surface-active: #1c2942;
    --track: #21262d;
    --accent: #4184e4;
    --accent-text: #0d1117;
    --ok: #3fb950;
    --warn: #d29922;
    --err: #f85149;
    background-color: #0d1117;
    color: var(--text);
  }
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  min-height: 100vh;
}

.app {
  max-width: 860px;
  margin: 0 auto;
  padding: 24px 20px 40px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.app-header h1 {
  margin: 0;
  font-size: 22px;
  letter-spacing: 0.5px;
}

.subtitle {
  margin: 2px 0 0;
  font-size: 12px;
  color: var(--text-dim);
}

h2 {
  margin: 0;
  font-size: 15px;
}

.queue {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.queue-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.queue-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.queue-hint {
  font-size: 12px;
  color: var(--text-dim);
}

.job-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.empty {
  padding: 40px 20px;
  text-align: center;
  border: 1px dashed var(--border);
  border-radius: 10px;
  color: var(--text);
}

.empty .muted {
  font-size: 12px;
  color: var(--text-dim);
}

.summary {
  margin: 0;
  font-size: 12px;
  color: var(--text-dim);
}

.queue-footer {
  display: flex;
  gap: 14px;
  font-size: 11px;
  color: var(--text-dim);
}

.btn {
  padding: 7px 16px;
  font-size: 13px;
  font-family: inherit;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--surface);
  color: var(--text);
  cursor: pointer;
  transition: border-color 0.15s, opacity 0.15s;
}

.btn:hover:not(:disabled) {
  border-color: var(--accent);
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn.primary {
  border-color: var(--accent);
  background: var(--accent);
  color: var(--accent-text);
  font-weight: 600;
}
</style>
