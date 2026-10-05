<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog";
import { addFiles } from "../stores/jobs";

const VIDEO_FILTER = [
  {
    name: "媒体文件",
    extensions: [
      "mp4", "mkv", "mov", "avi", "webm", "flv", "wmv", "mpg", "mpeg", "ts",
      "m4v", "3gp", "ogv", "gif", "mp3", "m4a", "aac", "flac", "wav", "ogg", "opus",
    ],
  },
];

async function pickFiles(): Promise<void> {
  const selected = await open({
    multiple: true,
    filters: VIDEO_FILTER,
  });
  if (!selected) return;
  const paths = Array.isArray(selected) ? selected : [selected];
  await addFiles(paths);
}
</script>

<template>
  <button type="button" class="add-btn" @click="pickFiles">＋ 添加文件</button>
</template>

<style scoped>
.add-btn {
  padding: 9px 18px;
  border: 1px solid var(--accent);
  border-radius: 8px;
  background: var(--accent);
  color: var(--accent-text);
  font-size: 13px;
  font-weight: 600;
  font-family: inherit;
  cursor: pointer;
  transition: opacity 0.15s;
}

.add-btn:hover {
  opacity: 0.88;
}
</style>
