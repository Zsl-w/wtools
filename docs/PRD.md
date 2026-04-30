# wTools 产品需求文档 (PRD)

## 1. 产品概述

### 1.1 产品定位
**wTools** 是一款极简的桌面应用与文件快速搜索工具，灵感源于 macOS Spotlight。秉承「呼之即来，即用即走」的设计理念，通过全局快捷键毫秒级唤起，帮助用户快速找到并打开应用程序和文件。

**核心依赖**：
- **应用搜索**：内置应用索引
- **文件搜索**：调用本地 [Everything](https://www.voidtools.com/) 服务

### 1.2 核心特性
- **极速响应**：毫秒级窗口唤起，流畅的动画体验
- **应用搜索**：快速定位并启动本地应用程序
- **文件搜索**：利用 Everything 实现毫秒级全盘文件搜索
- **全局快捷键**：一键唤起，无需鼠标操作
- **智能排序**：基于使用频率和时间的智能排序

### 1.3 设计理念
- **极简主义**：只做一件事，并做到极致
- **玻璃质感**：现代化 Glassmorphism 设计语言
- **直觉交互**：零学习成本，即用即会
- **极速体验**：搜索结果实时呈现，无需等待

---

## 2. 目标用户

### 2.1 主要用户群体
| 用户类型 | 特征 | 核心需求 |
|---------|------|---------|
| 效率工作者 | 开发者、设计师、产品经理 | 快速启动应用、查找项目文件 |
| 轻度办公用户 | 文员、学生、教师 | 快速打开软件、定位文档 |
| 键盘党 | 习惯使用键盘操作的用户 | 无需鼠标，快捷键完成所有操作 |

### 2.2 用户场景
1. **快速启动应用**：`Alt+Space` → 输入「chrome」→ 回车启动
2. **查找文件**：`Alt+Space` → 输入「报告」→ 找到并打开文档
3. **打开文件夹**：`Alt+Space` → 输入「下载」→ 定位到下载文件夹

---

## 3. 产品架构

### 3.1 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                      表现层 (Frontend)                       │
│  ┌──────────────────────────────────────────────────────┐  │
│  │                    搜索核心                            │  │
│  │   ┌─────────────┐   ┌─────────────┐   ┌───────────┐  │  │
│  │   │   搜索框     │   │  结果列表    │   │  状态栏    │  │  │
│  │   │  - 输入处理  │   │  - 应用结果  │   │  - 快捷键  │  │  │
│  │   │  - 实时匹配  │   │  - 文件结果  │   │  - 计数    │  │  │
│  │   └─────────────┘   └─────────────┘   └───────────┘  │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                      逻辑层 (Core)                           │
│  ┌─────────────────┐   ┌─────────────────┐   ┌───────────┐  │
│  │    搜索引擎      │   │   应用索引管理    │   │  配置管理  │  │
│  │   - 模糊匹配     │   │   - 应用扫描     │   │  - 偏好    │  │
│  │   - 权重排序     │   │   - 索引更新     │   │  - 历史    │  │
│  │   - 结果合并     │   │   - 图标提取     │   │  - 热键    │  │
│  └─────────────────┘   └─────────────────┘   └───────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                      系统层 (Tauri)                          │
│  ┌─────────────────┐   ┌─────────────────┐   ┌───────────┐  │
│  │    窗口管理      │   │    系统API       │   │  全局热键  │  │
│  │   - 显隐控制     │   │   - 应用扫描     │   │  - 注册    │  │
│  │   - 位置记忆     │   │   - Everything   │   │  - 监听    │  │
│  │   - 动画效果     │   │   - 执行启动     │   │  - 冲突    │  │
│  └─────────────────┘   └─────────────────┘   └───────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 模块职责

| 模块 | 职责 | 技术实现 |
|------|------|---------|
| 搜索核心 | 处理用户输入、合并应用和文件结果、排序渲染 | Fuse.js + 自定义权重算法 |
| 应用索引管理 | 本地应用扫描、索引构建、图标提取 | 后台定时扫描 + 注册表读取 |
| Everything 接口 | 调用 Everything SDK 搜索文件 | Everything DLL / CLI |
| 配置管理 | 用户偏好、搜索历史、热键设置 | Pinia + Tauri Store |

---

## 4. 功能规格

### 4.1 搜索核心

#### 4.1.1 主搜索框
- **触发方式**：全局快捷键（默认 `Alt+Space`）
- **输入框特性**：
  - 单行输入，自动聚焦
  - 占位符文案：「搜索应用或文件...」
  - 支持拼音搜索（如输入「js」匹配「记事本」），推荐使用 `pinyin-pro` 库预处理应用名称生成拼音索引
  - 支持首字母缩写（如输入「vx」匹配「Visual Studio Code」）（可选）
  - 支持模糊匹配

#### 4.1.2 搜索技术实现

| 功能 | 技术方案 | 说明 |
|------|---------|------|
| 模糊匹配 | Fuse.js | 英文及短拼音友好，内置权重算法 |
| 拼音匹配 | `pinyin-pro` | 预处理应用名称，生成拼音索引 |
| 备选拼音库 | `mkci`、`pinyin-match` | 轻量级替代方案 |

#### 4.1.3 搜索结果类型

| 类型 | 匹配项 | 图标 | 操作 |
|------|--------|------|------|
| 应用程序 | 本地安装的软件、UWP应用 | 应用图标 | 启动应用 |
| 文件 | 文档、图片、视频等 | 文件类型图标 | 打开文件 |
| 文件夹 | 本地文件夹 | 文件夹图标 | 在资源管理器中打开 |

#### 4.1.3 搜索排序算法

```
权重计算公式：
Score = BaseScore × MatchWeight × FrequencyWeight × RecencyWeight × TypeWeight

BaseScore: 基础匹配分数 (0-100)
MatchWeight: 匹配位置权重（开头匹配 1.5x > 包含匹配 1.0x）
FrequencyWeight: 使用频率权重 (1.0x - 2.0x)
RecencyWeight: 最近使用权重 (1.0x - 1.5x)
TypeWeight: 类型权重（应用 1.2x > 文件 1.0x）
```

#### 4.1.4 结果展示规则
- **最大结果数**：15 条（应用最多 8 条 + 文件最多 7 条）
- **分类显示**：应用和文件分组展示，应用优先
- **高亮匹配**：匹配的文字使用主色高亮
- **快捷编号**：前 9 条结果显示编号（1-9），支持 `Alt+数字` 快速打开

### 4.2 应用搜索

#### 4.2.1 应用索引范围
- Windows: 开始菜单、桌面快捷方式、注册表安装项
- macOS: /Applications、~/Applications
- Linux: /usr/share/applications、~/.local/share/applications

#### 4.2.2 索引更新策略
- **首次启动**：全量扫描，显示加载状态
- **定时更新**：每 30 分钟后台增量扫描
- **实时监听**：监控常见安装目录变化（可选，性能考虑）
- **手动刷新**：设置中提供「重新索引」按钮

### 4.3 文件搜索（基于 Everything）

#### 4.3.1 Everything 集成方式

wTools 通过以下方式调用本地 Everything 服务：

| 方式 | 优点 | 缺点 | 选用方案 |
|------|------|------|---------|
| **Everything SDK (DLL)** | 极速、功能完整 | 需处理 32/64 位兼容 | ✅ 主要方案 |
| **es.exe (CLI)** | 简单易用、跨架构 | 进程开销略高 | ✅ 备选方案 |
| **HTTP API (ETP)** | 可远程调用 | 需开启 ETP 服务器 | ❌ 暂不支持 |

#### 4.3.2 调用逻辑

```
用户输入关键词
     │
     ▼
┌─────────────────┐
│ 检查 Everything  │─── 未运行 ───► 提示用户安装/启动 Everything
│   是否运行       │
└─────────────────┘
     │
     ▼ 正在运行
┌─────────────────┐
│ 调用 Everything  │─── SDK: Everything_Search() / CLI: es.exe <query>
│   搜索接口       │
└─────────────────┘
     │
     ▼
┌─────────────────┐
│ 获取搜索结果     │─── 文件名、路径、类型、修改时间
│ (限制数量)       │
└─────────────────┘
     │
     ▼
┌─────────────────┐
│ 合并应用结果     │─── 按权重排序
│ 渲染展示         │
└─────────────────┘
```

#### 4.3.3 Everything 搜索参数

| 参数 | 说明 | 示例 |
|------|------|------|
| `-n <num>` | 限制结果数量 | `es.exe -n 10 报告` |
| `-s` | 按大小排序 | `es.exe -s 视频` |
| `-dm` | 按修改日期排序 | `es.exe -dm 最近` |
| `path:` | 限制路径 | `es.exe path:C:\Work 文档` |
| `ext:` | 限制扩展名 | `es.exe ext:pdf 合同` |

#### 4.3.4 搜索结果过滤

- **默认过滤**：排除系统目录（Windows、Program Files 等）
- **文件类型过滤**：可配置只搜索指定类型
- **路径过滤**：可配置排除特定目录
- **大小过滤**：排除超大文件（> 1GB）

### 4.4 搜索历史

- **历史记录**：保存最近 50 条搜索记录
- **快速访问**：输入框为空时显示最近使用
- **清空历史**：设置中提供清空选项

---

## 5. 用户界面设计

### 5.1 视觉风格

#### 5.1.1 设计语言：Glassmorphism
- **背景**：半透明毛玻璃效果（`backdrop-filter: blur(20px)`）
- **边框**：1px 半透明白色边框
- **阴影**：多层柔和投影营造层次感
- **圆角**：大圆角设计（16-24px）

#### 5.1.2 色彩系统

**主色调（TDesign 品牌蓝）**
```css
--primary-50: #EEF2FC;
--primary-100: #D9E5FC;
--primary-200: #B6CBF9;
--primary-300: #8AABF5;
--primary-400: #5C8AF0;
--primary-500: #0052D9;  /* 品牌主色 */
--primary-600: #003CAB;
--primary-700: #002A7D;
--primary-800: #001B4F;
--primary-900: #000F29;
```

**中性色**
```css
--gray-50: #F5F5F5;
--gray-100: #E8E8E8;
--gray-200: #D4D4D4;
--gray-300: #B9B9B9;
--gray-400: #9E9E9E;
--gray-500: #828282;
--gray-600: #666666;
--gray-700: #4D4D4D;
--gray-800: #333333;
--gray-900: #1A1A1A;
```

**玻璃效果**
```css
--glass-bg: rgba(255, 255, 255, 0.72);
--glass-border: rgba(255, 255, 255, 0.5);
--glass-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
--glass-blur: 20px;
```

#### 5.1.3 字体系统
- **英文字体**：Outfit (Google Font)
- **中文字体**：PingFang SC, Microsoft YaHei
- **等宽字体**：JetBrains Mono

| 级别 | 大小 | 字重 | 行高 |
|------|------|------|------|
| Display | 28px | 600 | 1.2 |
| H1 | 20px | 600 | 1.3 |
| H2 | 16px | 500 | 1.4 |
| Body | 14px | 400 | 1.5 |
| Caption | 12px | 400 | 1.4 |
| Small | 11px | 400 | 1.3 |

### 5.2 界面布局

#### 5.2.1 主窗口结构
```
┌─────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  🔍  搜索应用或文件...                           ✕     │   │  ← 搜索栏 (56px)
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  📱 应用程序 (5)                                         │   │  ← 分类标题
│  │  ┌──────────────────────────────────────────────────┐  │   │
│  │  │ 1  🌐  Google Chrome          chrome.exe         │  │   │  ← 结果项
│  │  │ 2  📝  Visual Studio Code     code.exe           │  │   │
│  │  │ 3  🎨  Figma                  Figma.exe          │  │   │
│  │  │ 4  ⚙️   设置                   System Settings   │  │   │
│  │  │ 5  🗂️   文件资源管理器          explorer.exe      │  │   │
│  │  └──────────────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  📄 文件 (4)                                             │   │
│  │  ┌──────────────────────────────────────────────────┐  │   │
│  │  │ 6  📊  2024年度总结报告.pptx    ~/Documents/      │  │   │
│  │  │ 7  📄  项目需求文档.docx        ~/Work/Projects/  │  │   │
│  │  │ 8  📝  会议纪要.md              ~/Notes/          │  │   │
│  │  │ 9  📁  下载                     ~/Downloads/      │  │   │
│  │  └──────────────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ─────────────────────────────────────────────────────────      │
│  ⚙️ 设置    ↑↓选择    Enter打开    Alt+数字快速打开    Esc关闭    │  ← 状态栏 (36px)
└─────────────────────────────────────────────────────────────────┘
```

#### 5.2.2 窗口尺寸
- **宽度**：680px（固定）
- **最小高度**：120px（仅搜索栏）
- **最大高度**：520px（展开结果，约 15 条结果）
- **位置**：屏幕中央偏上（距顶部 20%）

#### 5.2.3 搜索栏规格
- **高度**：56px
- **圆角**：28px（全圆角）
- **内边距**：0 24px
- **阴影**：`0 4px 20px rgba(0, 82, 217, 0.15)`
- **聚焦状态**：边框颜色变为 `--primary-500`，添加发光效果
- **图标**：左侧搜索图标 20px，右侧清除按钮（有输入时显示）

#### 5.2.4 结果项规格
- **高度**：48px
- **内边距**：12px 16px
- **圆角**：12px
- **布局**：Flex 布局
  - 序号/图标区域：32px
  - 名称区域：flex: 1
  - 路径/描述区域：固定宽度或省略
- **悬停效果**：背景色微变，添加选中指示器
- **选中状态**：左侧添加 3px 主色指示条，背景高亮

### 5.3 动画设计

#### 5.3.1 窗口动画
| 动作 | 动画 | 时长 | 缓动函数 |
|------|------|------|---------|
| 显示 | 缩放 + 淡入 | 200ms | `cubic-bezier(0.4, 0, 0.2, 1)` |
| 隐藏 | 缩放 + 淡出 | 150ms | `cubic-bezier(0.4, 0, 1, 1)` |

#### 5.3.2 内容动画
| 元素 | 动画 | 时长 | 缓动函数 |
|------|------|------|---------|
| 搜索栏聚焦 | 边框发光 + 轻微放大 | 200ms | ease-out |
| 结果项出现 | 交错淡入 | 80ms | ease-out |
| 结果项选中 | 背景色过渡 | 100ms | ease |
| 滚动 | 平滑滚动 | 150ms | ease-out |

### 5.4 暗黑模式

#### 5.4.1 暗黑配色
```css
[data-theme="dark"] {
  --bg-primary: rgba(28, 28, 30, 0.85);
  --glass-bg: rgba(30, 30, 32, 0.8);
  --glass-border: rgba(255, 255, 255, 0.1);
  --glass-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  --text-primary: #FFFFFF;
  --text-secondary: #98989F;
  --text-tertiary: #636366;
}
```

#### 5.4.2 自动切换
- 跟随系统主题自动切换
- 支持手动强制指定
- 过渡动画：200ms 平滑切换

---

## 6. 交互设计

### 6.1 全局快捷键

| 快捷键 | 功能 | 可自定义 |
|--------|------|---------|
| `Alt+Space` | 唤起/隐藏主窗口 | 是 |
| `Esc` | 关闭窗口 | 否 |
| `↑ / ↓` | 选择上/下一条结果 | 否 |
| `Enter` | 打开选中的项 | 否 |
| `Alt+数字` | 快速打开对应编号项 | 否 |
| `Ctrl/Cmd+Enter` | 在资源管理器中显示 | 否 |
| `Ctrl/Cmd+,` | 打开设置 | 是 |

### 6.2 鼠标交互

| 操作 | 响应 |
|------|------|
| 单击结果项 | 打开该项 |
| 右键结果项 | 显示上下文菜单（打开/打开所在位置/复制路径） |
| 单击搜索框外 | 关闭窗口（可配置） |
| 滚轮 | 滚动结果列表 |

### 6.3 上下文菜单

右键结果项显示：
- **打开**：启动应用或打开文件
- **打开所在位置**：在资源管理器中定位（文件/文件夹）
- **复制路径**：复制完整路径到剪贴板
- **固定到顶部**：将该项固定在搜索结果顶部

---

## 7. 技术架构

### 7.1 技术栈

| 层级 | 技术选型 | 说明 |
|------|---------|------|
| 桌面框架 | Tauri v2 | Rust 构建，轻量安全 |
| 前端框架 | Vue 3 + TypeScript | 组合式 API |
| UI 组件库 | TDesign Vue Next | 腾讯开源设计体系 |
| 状态管理 | Pinia | Vue 官方推荐 |
| 构建工具 | Vite | 极速构建 |
| 应用搜索 | Fuse.js | 应用名称模糊搜索 |
| 文件搜索 | Everything SDK | 调用 Everything DLL |
| 数据存储 | Tauri Store / SQLite | 本地配置和历史 |

### 7.2 项目结构

```
wtools/
├── src/
│   ├── assets/
│   │   ├── icons/
│   │   ├── fonts/
│   │   └── styles/
│   │       ├── global.css
│   │       ├── variables.css
│   │       └── animations.css
│   │
│   ├── components/
│   │   ├── SearchBar/
│   │   │   └── index.vue
│   │   ├── ResultList/
│   │   │   └── index.vue
│   │   ├── ResultItem/
│   │   │   └── index.vue
│   │   └── StatusBar/
│   │       └── index.vue
│   │
│   ├── views/
│   │   ├── MainWindow/
│   │   │   └── index.vue
│   │   └── Settings/
│   │       └── index.vue
│   │
│   ├── stores/
│   │   ├── settings.ts
│   │   ├── search.ts
│   │   └── history.ts
│   │
│   ├── composables/
│   │   ├── useSearch.ts
│   │   ├── useWindow.ts
│   │   ├── useShortcuts.ts
│   │   └── useAppIndex.ts
│   │
│   ├── utils/
│   │   ├── index.ts
│   │   ├── fuzzySearch.ts
│   │   └── formatters.ts
│   │
│   ├── types/
│   │   └── index.ts
│   │
│   ├── App.vue
│   └── main.ts
│
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/
│   │   │   ├── app.rs              # 应用搜索
│   │   │   ├── everything.rs       # Everything 接口
│   │   │   ├── window.rs           # 窗口控制
│   │   │   └── config.rs           # 配置管理
│   │   ├── everything/
│   │   │   ├── mod.rs              # Everything 模块
│   │   │   ├── sdk.rs              # SDK FFI 绑定
│   │   │   └── cli.rs              # CLI 封装
│   │   ├── indexer/
│   │   │   └── app_indexer.rs      # 应用索引
│   │   └── utils/
│   │       └── mod.rs
│   ├── Everything.dll              # Everything SDK (x64)
│   ├── Everything32.dll            # Everything SDK (x86)
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── docs/
├── design/
├── public/
├── index.html
├── package.json
├── tsconfig.json
└── vite.config.ts
```

### 7.3 核心 API 设计

#### 7.3.1 窗口控制 API
```typescript
// 唤起窗口
invoke('show_window')

// 隐藏窗口
invoke('hide_window')

// 设置窗口大小
invoke('set_window_size', { height: number })
```

#### 7.3.2 搜索 API
```typescript
// 获取已安装应用列表
invoke('get_installed_apps')

// 搜索文件（调用 Everything）
invoke('search_files_everything', { 
  query: string, 
  limit: number,
  filters?: {
    paths?: string[],      // 限制路径
    excludePaths?: string[], // 排除路径
    extensions?: string[]    // 文件扩展名
  }
})

// 启动应用
invoke('launch_app', { path: string })

// 打开文件
invoke('open_file', { path: string })

// 在资源管理器中显示
invoke('show_in_folder', { path: string })

// 复制路径到剪贴板
invoke('copy_to_clipboard', { text: string })
```

#### 7.3.3 Everything 状态 API
```typescript
// 检查 Everything 是否运行
invoke('is_everything_running')

// 检查 Everything 是否安装
invoke('is_everything_installed')

// 获取 Everything 版本
invoke('get_everything_version')

// 重建应用索引（Everything 不需要重建文件索引）
invoke('rebuild_app_index')

// 监听 Everything 状态变化
listen('everything_status_changed', (event) => { ... })
```

#### 7.3.4 Rust Everything SDK FFI 示例
```rust
// src-tauri/src/everything/sdk.rs
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint};
use std::ptr;

#[link(name = "Everything")]
extern "system" {
    fn Everything_SetSearchW(lpString: *const c_char);
    fn Everything_SetMax(max_results: c_uint);
    fn Everything_Query(bWait: c_int) -> c_int;
    fn Everything_GetNumResults() -> c_uint;
    fn Everything_GetResultFullPathNameW(nIndex: c_uint, buf: *mut c_char, bufsize: c_uint) -> c_uint;
    fn Everything_IsDBLoaded() -> c_int;
    fn Everything_GetLastError() -> c_uint;
}

pub fn search(query: &str, max_results: u32) -> Result<Vec<String>, String> {
    unsafe {
        // 设置搜索关键词 (Unicode)
        let c_query = CString::new(query).map_err(|e| e.to_string())?;
        Everything_SetSearchW(c_query.as_ptr());

        // 设置最大结果数
        Everything_SetMax(max_results);

        // 执行搜索
        if Everything_Query(1) == 0 {
            return Err(format!("Everything search failed: {}", Everything_GetLastError()));
        }

        // 获取结果数量
        let count = Everything_GetNumResults();
        let mut results = Vec::with_capacity(count as usize);

        // MAX_PATH = 260，使用动态buffer
        let buffer_size = 260usize;
        let mut buffer: Vec<u16> = vec![0u16; buffer_size];

        for i in 0..count {
            let len = Everything_GetResultFullPathNameW(
                i,
                buffer.as_mut_ptr() as *mut c_char,
                buffer_size as c_uint,
            );
            if len > 0 {
                let path = String::from_utf16_lossy(&buffer[..len as usize]);
                results.push(path);
            }
        }

        Ok(results)
    }
}
```

---

## 8. 性能指标

### 8.1 响应性能
| 指标 | 目标值 | 说明 |
|------|--------|------|
| 窗口唤起延迟 | < 80ms | 快捷键到窗口显示 |
| 应用搜索响应 | < 30ms | 应用名称匹配 |
| 文件搜索响应 | < 50ms | Everything 查询返回 |
| 首屏加载时间 | < 300ms | 应用启动到可用 |
| 应用索引构建 | < 3s | 首次扫描完成时间 |
| 内存占用 | < 80MB | 运行时内存（不含 Everything） |
| 安装包大小 | < 50MB | 打包后体积 |

### 8.2 Everything 依赖说明

| 项目 | 说明 |
|------|------|
| **Everything 要求** | 用户需自行安装 Everything（首次启动时检测并提示） |
| **服务依赖** | Everything 必须正在运行（支持 Everything Service 或普通模式） |
| **首次索引** | Everything 会自动建立文件索引，wTools 无需等待 |
| **实时性** | 文件搜索结果实时反映磁盘变化（由 Everything 保证） |

---

## 9. 设置选项

### 9.1 通用设置
- **开机启动**：是否随系统启动
- **失焦关闭**：点击窗口外是否自动关闭
- **主题**：跟随系统 / 浅色 / 深色

### 9.2 快捷键设置
- **唤起热键**：自定义唤起快捷键（默认 Alt+Space）
- **检测冲突**：自动检测并提示快捷键冲突

### 9.3 应用搜索设置
- **重新索引应用**：手动重建应用索引
- **索引状态**：显示当前应用索引条目数

### 9.4 Everything 设置
- **Everything 路径**：手动指定 es.exe 路径（可选）
- **搜索结果限制**：设置最大文件结果数（5-20）
- **文件类型过滤**：只搜索指定扩展名（可选）
- **路径过滤**：排除特定目录

---

## 10. 开发计划

### 10.1 里程碑

| 阶段 | 目标 | 时间 |
|------|------|------|
| M1 | 基础框架搭建，窗口管理，热键注册 | Week 1 |
| M2 | 应用索引，应用搜索，启动功能 | Week 2 |
| M3 | Everything 集成（SDK/CLI），文件搜索 | Week 3 |
| M4 | 搜索历史，智能排序，结果合并 | Week 4 |
| M5 | 设置界面，主题系统，暗黑模式 | Week 5 |
| M6 | 测试修复，打包发布 | Week 6 |

### 10.2 版本规划

**v0.1.0 (Alpha)**
- 基础窗口管理
- 应用搜索功能

**v0.5.0 (Beta)**
- Everything 文件搜索
- 搜索历史
- 基础设置

**v1.0.0 (Release)**
- 完整搜索功能
- 主题系统
- 性能优化
- 全平台支持

---

## 11. 附录

### 11.1 参考资源
- [Everything](https://www.voidtools.com/) - 文件搜索引擎
- [Everything SDK](https://www.voidtools.com/support/everything/sdk/) - SDK 文档
- [Spotlight - macOS](https://support.apple.com/guide/mac-help/search-with-spotlight-mchlp1008/mac)
- [TDesign Vue Next](https://tdesign.tencent.com/vue-next/)
- [Tauri](https://tauri.app/)
- [Vue 3](https://vuejs.org/)

### 11.2 设计原则
1. **专注单一**：只做搜索这一件事，做到极致
2. **极速体验**：每个操作都应该是毫秒级的
3. **键盘优先**：所有操作都可以用键盘完成
4. **简约美观**：去除一切不必要的元素
5. **稳定可靠**：依赖 Everything 保证文件搜索稳定性

### 11.3 术语表
| 术语 | 解释 |
|------|------|
| 唤起 | 通过快捷键显示主窗口 |
| Everything | Voidtools 开发的极速文件搜索工具 |
| SDK | Software Development Kit，软件开发工具包 |
| FFI | Foreign Function Interface，外部函数接口 |
| 权重 | 决定搜索结果排序的优先级 |

---

**文档版本**: v3.0  
**最后更新**: 2026-04-15  
**作者**: wTools Team
