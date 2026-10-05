# Third-Party Notices

本文件随 FFForge 分发包一并提供，用于满足 GPL 合规要求（对应源仓库
`docs/ffmpeg-gui-tech-stack.md` 第九节）。

## 1. FFForge（本应用）

- 许可证：**GPL-3.0-or-later**（见随包 `LICENSE`）
- 源代码：https://example.invalid/ffforge（发布时替换为实际公开仓库地址）
  发布说明中同时注明源码地址，以满足 GPL 对二进制分发须提供对应源码
  （或有效期不少于三年的书面要约）的要求。

## 2. FFmpeg / FFProbe

- 版本：`N-126342-gf88b741dbf-20260831`（master，2026-08-31 构建）
- 来源：BtbN FFmpeg-Builds
  https://github.com/BtbN/FFmpeg-Builds/releases （`ffmpeg-master-latest-win64-gpl.zip`，
  经由 winget 包 `BtbN.FFmpeg.GPL` 获取，本地下载日期 2026-10-03）
- 构建：**GPL**（`--enable-gpl --enable-version3`，完整编译选项见随包
  `FFMPEG-BUILD-INFO.txt`，其中含 SHA256 校验值）
- 许可证：FFmpeg 采用 **GPL-2.0-or-later**（部分组件为 LGPL；本构建为 GPL 组合）
- FFmpeg 源代码获取：
  - `https://git.ffmpeg.org/ffmpeg.git`
  - GitHub 镜像：`https://github.com/FFmpeg/FFmpeg`（对应提交 `gf88b741dbf`）
- 版权声明：FFmpeg 二进制与文档中的版权与许可证文本均完整保留，
  未做任何删改。

### 许可证兼容性说明

FFmpeg 的 "GPL version 2 or later" 与本应用的 GPL-3.0-or-later 双向兼容
（GPL-3.0 包含对 GPL-2.0-or-later 作品的升级路径）。本应用以子进程方式
调用 FFmpeg 并随包分发其 GPL 二进制，整个分发物按 GPL 提供；本项目已
开源，因此无冲突。

### Nonfree 组件核查

该构建 `--disable-libfdk-aac`，不含任何 `nonfree` 组件，**可合法再分发**。
音频转码使用原生 `aac` 编码器或 `libopus`。

## 3. 其他随包组件

- Tauri（Apache-2.0 / MIT）、Vue（MIT）、Vite（MIT）等仅编译进应用本体，
  其许可证按各自上游发布方式获取源码；随 release 附源码仓库链接即可。
