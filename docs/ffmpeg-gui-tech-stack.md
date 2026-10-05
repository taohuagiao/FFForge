# FFmpeg GUI — 技术选型说明

> 许可证决策：**开源**，使用 FFmpeg 的 **GPL 构建**（编码器完整，无功能阉割），应用自身采用 **GPL-3.0-or-later**。

目标：一个跨桌面平台的 FFmpeg 图形界面工具（选文件 → 选参数 → 转码 → 实时进度 → 可取消）。
本文件只规定**用什么技术、为什么、版本与约束**，不含实现代码。接手的 agent 按此选型自行实现。

## 一、结论

| 层 | 选型 | 版本 | 说明 |
|---|---|---|---|
| 桌面壳 | **Tauri 2** | 2.x | Rust 外壳 + 系统 WebView，包体约 10MB |
| 后端语言 | **Rust** | 1.77+ | 只负责进程管理，业务逻辑很浅 |
| 前端框架 | **Vue 3 + TypeScript** | Vue 3.5 / TS 5.7 | 开发者已熟悉；React 亦可平替 |
| 构建工具 | **Vite** | 6.x | Tauri 官方模板默认 |
| 包管理器 | **pnpm 或 npm** | — | 二选一，全项目统一 |
| 状态管理 | **Pinia**（或单个 reactive store） | 2.x | 任务队列规模小，二者皆可 |
| UI 样式 | **原生 CSS** | — | 首版不引入组件库，避免体积与版本包袱 |
| 外部二进制 | **ffmpeg / ffprobe 静态构建** | 6.x / 7.x | 以 sidecar 方式随包分发 |

备选：若团队不愿引入 Rust 工具链，换 **Electron + React**（同一套前端代码可复用，代价是包体 ~120MB、内存占用高）。

## 二、核心依赖清单

**Rust（`src-tauri/Cargo.toml`）**
- `tauri = "2"`
- `tauri-plugin-shell` — 以 sidecar 方式启动 ffmpeg/ffprobe
- `tauri-plugin-dialog` — 打开/保存文件对话框
- `tauri-plugin-fs` — 读写输出目录、校验路径存在性
- `serde` / `serde_json` — 命令与事件的数据结构
- `tokio`（`sync` + `macros` 特性）— 异步读进程输出流

**前端（`package.json`）**
- `@tauri-apps/api`、`@tauri-apps/cli`（均为 2.x）
- `@tauri-apps/plugin-dialog`、`@tauri-apps/plugin-shell`
- `vue`、`@vitejs/plugin-vue`、`vite`、`typescript`、`vue-tsc`

## 三、FFmpeg 的集成方式

1. **以子进程调用，不要链接 libav\***。GUI 只是"参数构造器 + 进程管理器 + 输出解析器"，链接 C 库带来的复杂度远超收益。
2. **二进制随包分发**：放入 `src-tauri/binaries/`，文件名必须带 target triple，例如
   `ffmpeg-x86_64-pc-windows-msvc.exe`、`ffprobe-x86_64-pc-windows-msvc.exe`，
   并在 `tauri.conf.json` 的 `bundle.externalBin` 中登记。
3. **二进制来源**：Windows 用 gyan.dev 的 essentials 构建，或 BtbN 的 nightly 构建。
4. **许可证（已定：开源，GPL 路线）**：使用 **GPL 构建**，编码器完整（libx264 / libx265 / libvpx / libopus 等），不受 LGPL 构建的功能限制。
   应用自身用 **GPL-3.0-or-later**，与 FFmpeg 的 "GPL v2 or later" 双向兼容。
   二进制来源二选一：
   - BtbN 的 `ffmpeg-master-latest-win64-gpl.zip`（包内即含 ffmpeg.exe / ffprobe.exe，许可证标注清晰）
   - gyan.dev 的 full build（下载前核对页面标注：essentials 为 GPLv2+，full 常见为 GPLv3）

   **红线**：禁止分发任何标注含 `nonfree` 组件（典型是 `libfdk_aac`）的构建，nonfree 构建不可再分发。
   合规细节见第九节。

## 四、关键设计约束（实现时不可绕过）

1. **进度解析**：启动时加 `-progress pipe:1 -nostats`，从 stdout 解析 `out_time_ms=` / `fps=` / `speed=` 这类 `key=value`；不要用 stderr 的 `frame=` 行。总时长先用 ffprobe 取 `format.duration` 才能算百分比。
2. **进度节流**：Rust 侧每 100ms 向前端推一次事件，逐行推送会把 IPC 和 UI 打满。
3. **参数以数组传递**：`Command::args([...])`，禁止拼字符串，否则中文路径、空格、`-vf` 表达式必炸。
4. **取消任务**：保存 `CommandChild` 句柄，调用 `kill()`；Windows 上注意 ffmpeg 不产生孙进程，直接 kill 即可。
5. **不弹黑框**：Windows 下 spawn 时需设置 `CREATE_NO_WINDOW` (0x08000000)，否则每次转码闪一个控制台窗口。

## 五、建议目录结构

```
ffmpeg-gui/
├── src/                     # Vue 前端
│   ├── lib/ipc.ts           # IPC 契约层：类型定义 + invoke/listen 封装
│   ├── stores/jobs.ts       # 任务队列状态
│   ├── components/          # 文件选择、任务行、进度条
│   └── App.vue
├── src-tauri/
│   ├── binaries/            # ffmpeg / ffprobe（带 target triple 后缀）
│   ├── capabilities/        # Tauri 2 权限配置
│   ├── src/
│   │   ├── ffmpeg.rs        # spawn、进度解析、取消
│   │   ├── probe.rs         # ffprobe 元数据探测
│   │   └── lib.rs / main.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
└── scripts/fetch-ffmpeg.ps1 # 下载并放置二进制
```

## 六、IPC 契约（前后端必须一致）

命令（前端 `invoke`）：

| 命令 | 入参 | 返回 |
|---|---|---|
| `probe` | `{ path }` | `{ durationMs, width, height, vcodec, acodec }` |
| `start_job` | `{ input, output, durationMs?, extraArgs[] }` | `jobId` |
| `cancel_job` | `{ id }` | `void` |

事件（前端 `listen`）：

| 事件 | 载荷 |
|---|---|
| `progress` | `{ id, outTimeMs, percent, fps, speed }` |
| `job-log` | `{ id, stream, line }` |
| `job-error` | `{ id, message }` |
| `job-finished` | `{ id, code, cancelled }` |

字段名前端用 camelCase，Rust 侧以 `#[serde(rename_all = "camelCase")]` 对齐。

## 七、开工检查清单

- [x] 许可证路线已定：开源 + FFmpeg GPL 构建，应用 GPL-3.0-or-later
- [ ] 下载时核对构建不含 nonfree 组件（有 `libfdk_aac` 就换一个）
- [ ] 安装 Rust 工具链（rustup）、Node 22 LTS、pnpm
- [ ] Windows 需安装 Visual Studio C++ 生成工具（MSVC）与 WebView2 运行时
- [ ] 下载 ffmpeg / ffprobe 并按 target triple 重命名放入 `src-tauri/binaries/`
- [ ] `npm run tauri dev` 跑通空壳，再接 ffmpeg
- [ ] 验收：转码一个带中文路径的文件 → 进度条平滑前进 → 中途取消能立刻终止 → 结束后无残留进程

## 八、FFmpeg 构建信息（需记录并随包分发）

下载二进制后，把以下信息写入 `src-tauri/binaries/FFMPEG-BUILD-INFO.txt` 并纳入版本库，GPL 合规要用：

```
ffmpeg -version          # 版本号 + 编译期 configure 参数
ffmpeg -buildconf        # 完整的 --enable-* 列表
来源 URL + 下载日期
SHA256 校验值
```

## 九、开源合规要点（GPL 路线）

1. **仓库根目录放 `LICENSE`**：GPL-3.0-or-later 全文。
2. **源码可得**：GPL 要求分发二进制时同时提供对应源码，或附带有效期不少于三年的书面要约。
   仓库公开 + release 里注明源码地址即可满足。
3. **随分发包附 `THIRD-PARTY-NOTICES.md`**：
   - FFmpeg 版本号、来源 URL、`configure` 配置
   - FFmpeg 源码获取方式（`https://git.ffmpeg.org/ffmpeg.git` 或 GitHub 镜像）
   - 明确写出本应用与 FFmpeg 各自的许可证
4. **保留版权声明**：不得删除 FFmpeg 二进制及其文档中的版权与许可证文本。
5. **不要混用 nonfree 组件**：含 `libfdk_aac` 的构建禁止再分发，需换用原生 `-c:a aac` 或 `libopus`。
6. **应用代码本身**：调用 FFmpeg 子进程不构成衍生作品的传染争议，但**随包分发** GPL 二进制意味着整个分发物需按 GPL 提供——项目已开源，因此无冲突。
