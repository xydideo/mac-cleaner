# 芒果清理 · mac-cleaner

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![GitHub](https://img.shields.io/badge/GitHub-xydideo%2Fmac--cleaner-24292f?logo=github)](https://github.com/xydideo/mac-cleaner)
[![Gitee](https://img.shields.io/badge/Gitee-xydideo%2Fmac--cleaner-c71d23?logo=gitee)](https://gitee.com/xydideo/mac-cleaner)

> 一款开源的 macOS 磁盘清理桌面应用。本地扫描、可视化管理、删除前确认，帮你安全释放磁盘空间。

**芒果清理** 基于 Tauri 2 + Vue 3 构建，原生 Mac 窗口，支持 Intel 与 Apple Silicon 通用包。默认只扫描不自动删除，适合日常维护与深度清理。

---

## 应用截图


| 概览                                                                                             | 大文件扫描                                                                                           |
| ---------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| ![概览](https://raw.giteeusercontent.com/xydideo/xydideo-imgs/raw/master/imgs/mac-cleaner/1.png) | ![大文件](https://raw.giteeusercontent.com/xydideo/xydideo-imgs/raw/master/imgs/mac-cleaner/2.png) |



| 应用卸载                                                                                             | 目录浏览                                                                                             |
| ------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| ![应用卸载](https://raw.giteeusercontent.com/xydideo/xydideo-imgs/raw/master/imgs/mac-cleaner/3.png) | ![目录浏览](https://raw.giteeusercontent.com/xydideo/xydideo-imgs/raw/master/imgs/mac-cleaner/4.png) |


---



## 功能介绍



### 概览

- 磁盘使用圆环与容量统计（总容量 / 已用 / 可用）
- **智能扫描**：快速扫描缓存、日志、废纸篓等常见垃圾
- **深度扫描**：在智能扫描基础上，额外扫描系统遗留与大文件线索
- 扫描过程支持暂停 / 继续，一键跳转清理模块



### 垃圾清理

- 按类别展示扫描结果（缓存、日志、废纸篓、系统遗留等）
- 勾选后批量删除，支持**移到废纸篓**或**永久删除**
- 删除前多重确认，系统保护路径不可误删



### 大文件

- 全盘扫描超过阈值的大文件与大文件夹
- 可配置文件 / 文件夹体积阈值（设置页）
- 显示所属应用标签，支持暂停扫描、结束并保留已扫结果
- 支持在访达中定位、批量清理



### 应用卸载（APP）

- 列出已安装应用及体积，按大小排序
- 扫描应用关联文件（缓存、配置、残留等）
- 完整卸载 App 及其关联数据



### 目录

- 拖入文件夹或选择目录，树形浏览与体积分析
- 快捷入口：主目录、下载、文稿、资源库等
- 图片文件显示缩略图，支持进入子目录与**返回上一级**
- 文件夹精确统计、批量删除、访达定位



### 其他

- 菜单栏托盘图标，快速显示 / 退出
- 设置页：删除方式、长期未动阈值、大文件阈值等
- 一键打开系统「完全磁盘访问」隐私设置

---



## 安装



### 系统要求

- macOS 10.13 及以上
- Intel 或 Apple Silicon（M 系列）Mac



### 下载安装

任选一种方式获取 **`芒果清理.dmg`**：

**方式一：仓库内直接下载（最简单）**

克隆或打开仓库后，进入 [`apps/`](apps/) 目录，下载其中的 `芒果清理.dmg`：

- GitHub：https://github.com/xydideo/mac-cleaner/tree/main/apps
- Gitee：https://gitee.com/xydideo/mac-cleaner/tree/main/apps

**方式二：从 Releases 下载（推荐，带版本号）**

- GitHub：https://github.com/xydideo/mac-cleaner/releases
- Gitee：https://gitee.com/xydideo/mac-cleaner/releases

**安装步骤**

1. 双击打开 DMG，将 **芒果清理** 拖入「应用程序」文件夹
2. 首次打开若提示「无法验证开发者」：右键应用 → **打开** → 确认即可

---



## 安全原则

- **纯本地运行**：所有扫描、分析、删除均在本地完成，不上传文件内容，不采集、不存储用户个人信息或使用行为
- **只扫不删**：默认仅扫描并列出，不会自动删除任何文件
- **删除前确认**：批量删除与永久删除均有二次确认
- **可恢复优先**：默认移到废纸篓，误删可恢复
- **路径保护**：系统关键目录纳入保护，避免误伤

---



## 从源码构建

适合开发者与贡献者本地调试、自行打包。

### 环境要求

- Node.js ≥ 18
- pnpm
- Rust 工具链（`rustup`）
- Xcode Command Line Tools（macOS）



### 开发运行

```bash
pnpm install
pnpm dev
```

> `pnpm dev` 会启动 Tauri 开发窗口。请在 **Mac 原生窗口** 中使用完整功能。

### 打包正式版

```bash
pnpm build
```

`pnpm build` 会构建 **Intel + Apple Silicon 通用包**，并自动复制到 **`apps/芒果清理.dmg`**。

### 其他命令

| 命令 | 说明 |
|------|------|
| `pnpm dev` | 启动 Tauri 开发环境 |
| `pnpm build` | 打包并输出 `apps/芒果清理.dmg` |
| `pnpm build:web` | 仅构建前端 `dist/` |
| `pnpm sync-icons` | 同步 Dock / 托盘 / UI 图标 |


---



## 技术栈


| 层级  | 技术                                             |
| --- | ---------------------------------------------- |
| 桌面壳 | [Tauri 2](https://v2.tauri.app/)               |
| 前端  | Vue 3 · TypeScript · Element Plus · Vue Router |
| 后端  | Rust（磁盘扫描、目录浏览、应用卸载等）                          |
| 构建  | Vite · pnpm                                    |


---



## 开源与贡献

本项目以 **MIT** 协议开源，欢迎：

- 提交 Issue 反馈 Bug 或建议
- 提交 Pull Request 改进功能与文档
- Star 支持项目传播

### 仓库地址

| 平台 | 地址 |
|------|------|
| GitHub | https://github.com/xydideo/mac-cleaner |
| Gitee | https://gitee.com/xydideo/mac-cleaner |

### 克隆项目

```bash
# GitHub
git clone https://github.com/xydideo/mac-cleaner.git

# Gitee（国内推荐）
git clone https://gitee.com/xydideo/mac-cleaner.git
```

### 维护者：如何发布安装包

日常发版流程：`pnpm build` → 产物自动复制到 **`apps/芒果清理.dmg`**。你可以两种方式让用户拿到安装包：

| 方式 | 适合场景 | 说明 |
|------|----------|------|
| 提交到 `apps/` | 快速、省事 | 把 dmg 随代码 push，用户直接在仓库里下载 |
| 上传到 Releases | 正式版本、历史版本管理 | 每个版本单独归档，带版本号和更新说明 |

**Gitee 也有 Releases**，中文界面里叫 **「发行版」**，功能和 GitHub Releases 类似。

#### GitHub 发 Release

1. 打开 https://github.com/xydideo/mac-cleaner → **Releases** → **Draft a new release**
2. **Choose a tag**：新建标签，如 `v0.1.0`（选 *Create new tag*）
3. **Release title**：如 `v0.1.0 首个开源版本`
4. **Describe this release**：写更新说明（支持 Markdown）
5. 在 **Attach binaries** 区域拖入 `apps/芒果清理.dmg`
6. 点 **Publish release**

#### Gitee 发发行版

1. 打开 https://gitee.com/xydideo/mac-cleaner → **发行版** → **创建发行版**
2. **版本号**：如 `v0.1.0`（可勾选「基于 Tag 创建」）
3. **发行版标题 / 说明**：填写版本描述
4. **上传附件**：选择 `apps/芒果清理.dmg`
5. 点 **创建**

> 建议：平时把最新包放在 `apps/` 方便直接下载；每次正式发版时，**同时**上传到两个平台的 Releases/发行版，方便用户按版本回溯。

---



## 作者

**xydideo** · [GitHub](https://github.com/xydideo) · [Gitee](https://gitee.com/xydideo)

## 许可证

[MIT](LICENSE)