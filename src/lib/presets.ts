// 转码预设：label + 输出扩展名 + ffmpeg 参数（数组，绝不拼字符串）。
export interface Preset {
  id: string;
  label: string;
  description: string;
  ext: string; // 输出扩展名（含点）
  args: string[];
  custom?: boolean; // 自定义预设：参数由用户输入
}

export const PRESETS: Preset[] = [
  {
    id: "h264",
    label: "MP4 · H.264",
    description: "兼容性最好，几乎所有设备可播",
    ext: ".mp4",
    args: ["-c:v", "libx264", "-preset", "medium", "-crf", "23", "-c:a", "aac", "-b:a", "128k"],
  },
  {
    id: "h265",
    label: "MP4 · H.265",
    description: "更省空间，适合长期保存",
    ext: ".mp4",
    args: ["-c:v", "libx265", "-preset", "medium", "-crf", "28", "-c:a", "aac", "-b:a", "128k"],
  },
  {
    id: "vp9",
    label: "WebM · VP9",
    description: "开源编码，适合网页嵌入",
    ext: ".webm",
    args: ["-c:v", "libvpx-vp9", "-crf", "30", "-b:v", "0", "-c:a", "libopus"],
  },
  {
    id: "remux",
    label: "重封装 · 不转码",
    description: "流复制到 MP4，速度极快",
    ext: ".mp4",
    args: ["-c", "copy"],
  },
  {
    id: "mp3",
    label: "提取音频 · MP3",
    description: "只保留音轨",
    ext: ".mp3",
    args: ["-vn", "-c:a", "libmp3lame", "-q:a", "2"],
  },
  {
    id: "custom",
    label: "自定义参数",
    description: "手动输入 ffmpeg 参数",
    ext: ".mp4",
    args: [],
    custom: true,
  },
];

export function presetById(id: string): Preset {
  return PRESETS.find((p) => p.id === id) ?? PRESETS[0];
}

/**
 * 由输入路径与预设推导输出路径：
 * 同目录、原名 + 预设后缀 + 新扩展名。
 * 例：/a/视频.mp4 + h264 → /a/视频.h264.mp4
 */
export function deriveOutputPath(input: string, preset: Preset): string {
  const sep = input.includes("\\") ? "\\" : "/";
  const idx = input.lastIndexOf(sep);
  const dir = idx >= 0 ? input.slice(0, idx + 1) : "";
  const name = idx >= 0 ? input.slice(idx + 1) : input;
  const dot = name.lastIndexOf(".");
  const stem = dot > 0 ? name.slice(0, dot) : name;
  return `${dir}${stem}.${preset.id}${preset.ext}`;
}

/** 自定义参数按空白切分为数组（与 Rust 侧数组传参对齐）。 */
export function splitArgs(text: string): string[] {
  return text.trim().split(/\s+/).filter(Boolean);
}

export function formatDuration(ms: number): string {
  if (!Number.isFinite(ms) || ms <= 0) return "--:--";
  const total = Math.floor(ms / 1000);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  const mm = String(m).padStart(2, "0");
  const ss = String(s).padStart(2, "0");
  return h > 0 ? `${h}:${mm}:${ss}` : `${m}:${ss}`;
}
