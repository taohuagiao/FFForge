# FFForge — FFmpeg 图形界面转码工具

跨桌面平台的 FFmpeg GUI：选文件 → 选参数 → 转码 → 实时进度 → 可取消。

技术选型依据见 [docs/ffmpeg-gui-tech-stack.md](docs/ffmpeg-gui-tech-stack.md)。
开源（GPL-3.0-or-later），随包分发 FFmpeg **GPL 构建**（无 nonfree 组件）。

## 开发环境要求

- Rust 1.77+（rustup，MSVC 工具链）
- Node 22 LTS + pnpm
- Windows：Visual Studio C++ 生成工具（MSVC）+ WebView2 运行时

## 运行

```bash
pnpm install
pnpm tauri dev
```

## 构建

```bash
pnpm tauri build
```

产物在 `src-tauri/target/release/bundle/`；ffmpeg/ffprobe 以 sidecar 方式
随包分发（`src-tauri/binaries/`，target triple 后缀已按 Tauri 约定命名）。

## 更新内置 FFmpeg

```powershell
powershell -ExecutionPolicy Bypass -File scripts/fetch-ffmpeg.ps1
```

脚本会下载 BtbN GPL 构建、校验不含 nonfree 组件、写入
`src-tauri/binaries/FFMPEG-BUILD-INFO.txt`（GPL 合规记录）。

## 结构

```
src/                     # Vue 3 前端
├── lib/ipc.ts           # IPC 契约层：类型 + invoke/listen 封装
├── lib/presets.ts       # 转码预设与输出路径推导
├── stores/jobs.ts       # 任务队列状态（单个 reactive store）
├── components/          # 预设选择 / 添加文件 / 任务行
└── App.vue
src-tauri/
├── binaries/            # ffmpeg / ffprobe sidecar + 构建信息记录
├── capabilities/        # Tauri 2 权限配置
└── src/
    ├── ffmpeg.rs        # spawn、-progress 解析、100ms 节流、取消
    ├── probe.rs         # ffprobe 元数据探测
    └── binaries.rs      # sidecar 路径解析 + CREATE_NO_WINDOW
```

## IPC 契约

命令：`probe` / `start_job` / `cancel_job`；
事件：`progress`（100ms 节流）/ `job-log` / `job-error` / `job-finished`。
字段 camelCase，与 Rust 侧 `#[serde(rename_all = "camelCase")]` 对齐。
详见 `src/lib/ipc.ts` 与 `docs/ffmpeg-gui-tech-stack.md` 第六节。

## 合规

- `LICENSE`：GPL-3.0-or-later
- `THIRD-PARTY-NOTICES.md`：FFmpeg 版本、来源、许可证与兼容性说明
- `src-tauri/binaries/FFMPEG-BUILD-INFO.txt`：构建配置 + SHA256

红线：禁止分发含 nonfree 组件（如 libfdk_aac）的 FFmpeg 构建。
