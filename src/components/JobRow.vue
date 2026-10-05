<script setup lang="ts">
import { computed } from "vue";
import {
  requestCancel,
  removeJob,
  statusLabel,
  type Job,
} from "../stores/jobs";
import { formatDuration } from "../lib/presets";

const props = defineProps<{ job: Job; index: number }>();

const fileName = computed(() => {
  const sep = props.job.input.includes("\\") ? "\\" : "/";
  return props.job.input.split(sep).pop() ?? props.job.input;
});

const percent = computed(() =>
  props.job.percent == null ? 0 : Math.min(100, Math.max(0, props.job.percent)),
);

const statusClass = computed(() => props.job.status);
const canCancel = computed(() => props.job.status === "running");
const canRemove = computed(() => props.job.status !== "running");
const showLog = computed(() => props.job.status === "failed" || props.job.status === "cancelled");
</script>

<template>
  <article class="job-row" :class="statusClass">
    <div class="job-main">
      <div class="job-title">
        <span class="file-name" :title="job.input">{{ fileName }}</span>
        <span class="badge" :class="statusClass">{{ statusLabel(job.status) }}</span>
      </div>
      <div class="job-meta">
        <span :title="job.output">→ {{ job.output || "…" }}</span>
        <span v-if="job.durationMs">{{ formatDuration(job.durationMs) }}</span>
        <span v-if="job.fps != null">{{ job.fps.toFixed(1) }} fps</span>
        <span v-if="job.speed != null">{{ job.speed.toFixed(2) }}×</span>
        <span v-if="job.status === 'running'">{{ formatDuration(job.outTimeMs) }} / {{ formatDuration(job.durationMs ?? 0) }}</span>
      </div>

      <div class="progress-track">
        <div class="progress-fill" :class="statusClass" :style="{ width: percent + '%' }" />
      </div>

      <p v-if="job.error" class="job-error" :title="job.error">{{ job.error }}</p>
      <pre v-if="showLog && job.log.length" class="job-log">{{ job.log.slice(-8).join("\n") }}</pre>
    </div>

    <div class="job-actions">
      <button
        v-if="canCancel"
        type="button"
        class="btn cancel"
        @click="requestCancel(job)"
      >
        取消
      </button>
      <button
        v-if="canRemove"
        type="button"
        class="btn remove"
        @click="removeJob(index)"
      >
        移除
      </button>
    </div>
  </article>
</template>

<style scoped>
.job-row {
  display: flex;
  gap: 12px;
  padding: 12px 14px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--surface);
}

.job-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.job-title {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.file-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.badge {
  flex-shrink: 0;
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 99px;
  border: 1px solid var(--border);
  color: var(--text-dim);
}

.badge.running {
  color: var(--accent);
  border-color: var(--accent);
}

.badge.done {
  color: var(--ok);
  border-color: var(--ok);
}

.badge.failed {
  color: var(--err);
  border-color: var(--err);
}

.badge.cancelled {
  color: var(--warn);
  border-color: var(--warn);
}

.job-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  font-size: 11px;
  color: var(--text-dim);
}

.job-meta span {
  max-width: 100%;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.progress-track {
  height: 6px;
  border-radius: 3px;
  background: var(--track);
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: 3px;
  background: var(--accent);
  transition: width 0.12s linear;
}

.progress-fill.done {
  background: var(--ok);
}

.progress-fill.failed {
  background: var(--err);
}

.progress-fill.cancelled {
  background: var(--warn);
}

.job-error {
  margin: 0;
  font-size: 11px;
  color: var(--err);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.job-log {
  margin: 0;
  font-size: 10px;
  line-height: 1.5;
  color: var(--text-dim);
  font-family: ui-monospace, Consolas, monospace;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 110px;
  overflow: auto;
}

.job-actions {
  display: flex;
  flex-direction: column;
  gap: 6px;
  justify-content: center;
}

.btn {
  padding: 6px 14px;
  font-size: 12px;
  font-family: inherit;
  border-radius: 6px;
  cursor: pointer;
  border: 1px solid var(--border);
  background: var(--surface);
  color: var(--text);
  transition: border-color 0.15s;
}

.btn.cancel {
  border-color: var(--warn);
  color: var(--warn);
}

.btn.remove {
  color: var(--text-dim);
}

.btn:hover {
  border-color: var(--accent);
}
</style>
