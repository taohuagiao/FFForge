// IPC 契约层：类型定义 + invoke/listen 封装。
// 前后端字段名以 camelCase 对齐（Rust 侧 #[serde(rename_all = "camelCase")]）。
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// ---------- 命令入参/返回 ----------

export interface ProbeResult {
  durationMs: number;
  width: number | null;
  height: number | null;
  vcodec: string | null;
  acodec: string | null;
}

export interface StartJobArgs {
  input: string;
  output: string;
  durationMs?: number;
  extraArgs: string[];
}

// ---------- 事件载荷 ----------

export interface ProgressEvent {
  id: number;
  outTimeMs: number;
  percent: number | null;
  fps: number | null;
  speed: number | null;
}

export interface JobLogEvent {
  id: number;
  stream: string;
  line: string;
}

export interface JobErrorEvent {
  id: number;
  message: string;
}

export interface JobFinishedEvent {
  id: number;
  code: number;
  cancelled: boolean;
}

// ---------- 命令封装 ----------

export function probeMedia(path: string): Promise<ProbeResult> {
  return invoke<ProbeResult>("probe", { path });
}

export function startJob(args: StartJobArgs): Promise<number> {
  return invoke<number>("start_job", { args });
}

export function cancelJob(id: number): Promise<void> {
  return invoke<void>("cancel_job", { id });
}

// ---------- 事件订阅封装 ----------

export function onProgress(cb: (e: ProgressEvent) => void): Promise<UnlistenFn> {
  return listen<ProgressEvent>("progress", (ev) => cb(ev.payload));
}

export function onJobLog(cb: (e: JobLogEvent) => void): Promise<UnlistenFn> {
  return listen<JobLogEvent>("job-log", (ev) => cb(ev.payload));
}

export function onJobError(cb: (e: JobErrorEvent) => void): Promise<UnlistenFn> {
  return listen<JobErrorEvent>("job-error", (ev) => cb(ev.payload));
}

export function onJobFinished(cb: (e: JobFinishedEvent) => void): Promise<UnlistenFn> {
  return listen<JobFinishedEvent>("job-finished", (ev) => cb(ev.payload));
}
