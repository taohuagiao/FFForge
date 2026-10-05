<script setup lang="ts">
import { computed } from "vue";
import { PRESETS } from "../lib/presets";
import { store } from "../stores/jobs";
import { splitArgs } from "../lib/presets";

const preset = computed(() => PRESETS.find((p) => p.id === store.presetId) ?? PRESETS[0]);
const customArgsPreview = computed(() =>
  preset.value.custom ? splitArgs(store.customArgs) : preset.value.args,
);
</script>

<template>
  <section class="preset-picker">
    <div class="preset-grid">
      <button
        v-for="p in PRESETS"
        :key="p.id"
        type="button"
        class="preset-card"
        :class="{ active: store.presetId === p.id }"
        @click="store.presetId = p.id"
      >
        <span class="preset-label">{{ p.label }}</span>
        <span class="preset-desc">{{ p.description }}</span>
      </button>
    </div>

    <div v-if="preset?.custom" class="custom-args">
      <label for="custom-ext">输出扩展名</label>
      <input
        id="custom-ext"
        v-model="store.customExt"
        class="ext-input"
        placeholder=".mp4"
      />
      <label for="custom-args">ffmpeg 参数（空格分隔，逐个作为数组元素传给后端）</label>
      <input
        id="custom-args"
        v-model="store.customArgs"
        class="args-input"
        placeholder="例如：-c:v libx264 -crf 20 -vf scale=1280:-2"
      />
    </div>

    <p class="args-preview">
      参数预览：
      <code v-if="customArgsPreview.length">{{ customArgsPreview.join(" ") }}</code>
      <code v-else class="muted">（空）</code>
    </p>
  </section>
</template>

<style scoped>
.preset-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 8px;
}

.preset-card {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--surface);
  cursor: pointer;
  text-align: left;
  font-family: inherit;
  transition: border-color 0.15s, background 0.15s;
}

.preset-card:hover {
  border-color: var(--accent);
}

.preset-card.active {
  border-color: var(--accent);
  background: var(--surface-active);
}

.preset-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
}

.preset-desc {
  font-size: 11px;
  color: var(--text-dim);
}

.custom-args {
  margin-top: 12px;
  display: grid;
  gap: 6px;
}

.custom-args label {
  font-size: 12px;
  color: var(--text-dim);
}

.ext-input {
  width: 100px;
}

.args-input {
  width: 100%;
  font-family: ui-monospace, Consolas, monospace;
  font-size: 12px;
}

.ext-input,
.args-input {
  padding: 7px 10px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--surface);
  color: var(--text);
  outline: none;
}

.ext-input:focus,
.args-input:focus {
  border-color: var(--accent);
}

.args-preview {
  margin: 10px 0 0;
  font-size: 11px;
  color: var(--text-dim);
  word-break: break-all;
}

.args-preview code {
  font-family: ui-monospace, Consolas, monospace;
  color: var(--accent);
}

.args-preview code.muted {
  color: var(--text-dim);
}
</style>
