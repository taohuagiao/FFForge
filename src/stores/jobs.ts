// 任务队列状态：单个 reactive store（任务规模小，无需 Pinia）。
import { reactive } from "vue";
import {
  cancelJob as ipcCancelJob,
  startJob as ipcStartJob,
  probeMedia,
  type ProbeResult,
} from "../lib/ipc";
import { deriveOutputPath, presetById } from "../lib/presets";

export type JobStatus = "pending" | "probing" | "running" | "done" | "failed" | "cancelled";

export interface Job {
  /** Rust 侧分配的任务 id；入队/探测阶段为 null。 */
  id: number | null;
  input: string;
  output: string;
  presetId: string;
  extraArgs: string[];
  status: JobStatus;
  percent: number | null;
  outTimeMs: number;
  durationMs: number | null;
  fps: number | null;
  speed: number | null;
  error: string | null;
  log: string[];
}

interface JobsState {
  jobs: Job[];
  /** 当前 presetId（对新增文件生效）。 */
  presetId: string;
  customArgs: string;
  customExt: string;
  /** 顺序执行：同时只跑一个 ffmpeg。 */
  running: boolean;
}

export const store = reactive<JobsState>({
  jobs: [],
  presetId: "h264",
  customArgs: "",
  customExt: ".mp4",
  running: false,
});

function jobByIpcId(id: number): Job | undefined {
  return store.jobs.find((j) => j.id === id);
}

export function statusLabel(status: JobStatus): string {
  switch (status) {
    case "pending": return "排队中";
    case "probing": return "读取信息";
    case "running": return "转码中";
    case "done": return "完成";
    case "failed": return "失败";
    case "cancelled": return "已取消";
  }
}

/** 添加文件：探测元数据（时长用于百分比计算），然后进入待转码队列。 */
export async function addFiles(paths: string[]): Promise<void> {
  const preset = presetById(store.presetId);
  for (const path of paths) {
    const job: Job = reactive({
      id: null,
      input: path,
      output: "",
      presetId: store.presetId,
      extraArgs: preset.custom
        ? store.customArgs.trim().split(/\s+/).filter(Boolean)
        : [...preset.args],
      status: "probing",
      percent: null,
      outTimeMs: 0,
      durationMs: null,
      fps: null,
      speed: null,
      error: null,
      log: [],
    });
    store.jobs.push(job);
    try {
      const info: ProbeResult = await probeMedia(path);
      job.durationMs = info.durationMs;
    } catch (e) {
      job.status = "failed";
      job.error = `无法读取文件信息：${e}`;
      continue;
    }
    job.output = deriveOutputPath(
      job.input,
      preset.custom
        ? { ...preset, ext: store.customExt.trim() || ".mp4" }
        : preset,
    );
    job.status = "pending";
  }
}

export function removeJob(index: number): void {
  const job = store.jobs[index];
  if (job && job.status === "running" && job.id != null) {
    ipcCancelJob(job.id).catch(() => {});
  }
  store.jobs.splice(index, 1);
}

export function clearFinished(): void {
  store.jobs = store.jobs.filter(
    (j) => j.status === "pending" || j.status === "probing" || j.status === "running",
  );
}

export async function requestCancel(job: Job): Promise<void> {
  if (job.id != null) {
    try {
      await ipcCancelJob(job.id);
    } catch {
      /* 任务可能刚结束，忽略 */
    }
  }
}

/** 顺序启动所有 pending 任务（同时只跑一个 ffmpeg，避免抢满 CPU）。 */
export async function startAll(): Promise<void> {
  if (store.running) return;
  store.running = true;
  try {
    for (;;) {
      const next = store.jobs.find((j) => j.status === "pending");
      if (!next) break;
      try {
        const id = await ipcStartJob({
          input: next.input,
          output: next.output,
          durationMs: next.durationMs ?? undefined,
          extraArgs: next.extraArgs,
        });
        next.id = id;
        next.status = "running";
        next.percent = 0;
        next.outTimeMs = 0;
        // 等待该任务结束（job-finished 事件会把它移出 running）。
        await new Promise<void>((resolve) => {
          const timer = setInterval(() => {
            if (next.status !== "running") {
              clearInterval(timer);
              resolve();
            }
          }, 100);
        });
      } catch (e) {
        next.status = "failed";
        next.error = String(e);
      }
    }
  } finally {
    store.running = false;
  }
}

// ---------- 事件处理（由 App.vue 挂载时订阅） ----------

export function handleProgress(id: number, p: {
  outTimeMs: number;
  percent: number | null;
  fps: number | null;
  speed: number | null;
}): void {
  const job = jobByIpcId(id);
  if (!job || job.status !== "running") return;
  job.outTimeMs = p.outTimeMs;
  job.percent = p.percent;
  job.fps = p.fps;
  job.speed = p.speed;
}

export function handleJobLog(id: number, line: string): void {
  const job = jobByIpcId(id);
  if (!job) return;
  job.log.push(line);
  if (job.log.length > 200) job.log.shift();
}

export function handleJobError(id: number, message: string): void {
  const job = jobByIpcId(id);
  if (!job) return;
  job.error = message;
}

export function handleJobFinished(id: number, code: number, cancelled: boolean): void {
  const job = jobByIpcId(id);
  if (!job) return;
  if (cancelled) {
    job.status = "cancelled";
  } else if (code === 0) {
    job.status = "done";
    job.percent = 100;
  } else {
    job.status = "failed";
    if (!job.error) {
      job.error = `ffmpeg 退出码 ${code}`;
    }
  }
}
