# wTools UI 设计规范

## 概述

本文档定义了 wTools 的视觉设计系统，wTools 是一款极简的桌面应用与文件快速搜索工具。

---

## 设计原则

### 1. Glassmorphism（玻璃拟态）
- 半透明背景配合背景模糊效果
- 细腻的边框和阴影营造层次感
- 清新、现代的视觉风格

### 2. 极简主义
- 只做搜索一件事
- 去除不必要的装饰元素
- 充足的留白空间

### 3. 一致性
- 统一的色彩、字体、间距
- 可预测的用户体验
- 组件样式复用

---

## 色彩系统

### 主色调

```css
:root {
  /* Primary - Teal 青色 */
  --primary-50: #F0FDFA;
  --primary-100: #CCFBF1;
  --primary-200: #99F6E4;
  --primary-300: #5EEAD4;
  --primary-400: #2DD4BF;
  --primary-500: #14B8A6;  /* 主色 */
  --primary-600: #0D9488;
  --primary-700: #0F766E;
  --primary-800: #115E59;
  --primary-900: #134E4A;

  /* Accent - 品牌色语义化变体 */
  --accent: #14B8A6;
  --accent-light: #5EEAD4;
  --accent-subtle: rgba(20, 184, 166, 0.12);
  --accent-glow: rgba(20, 184, 166, 0.15);
  --selection-bg: rgba(20, 184, 166, 0.12);
  --selection-border: rgba(20, 184, 166, 0.3);
  --highlight-bg: rgba(20, 184, 166, 0.2);
  --highlight-text: #0D9488;

  /* Neutral - 中性灰 */
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

  /* Semantic - 语义色 */
  --success: #2BA471;
  --warning: #E37318;
  --error: #D54941;
  --info: #0052D9;
}
```

### 玻璃效果变量

```css
:root {
  /* Light Mode */
  --glass-bg: rgba(255, 255, 255, 0.72);
  --glass-bg-hover: rgba(255, 255, 255, 0.85);
  --glass-border: rgba(255, 255, 255, 0.5);
  --glass-shadow: 0 25px 50px rgba(0, 0, 0, 0.15), 0 0 0 1px rgba(255, 255, 255, 0.3);
  --glass-blur: blur(40px) saturate(180%);

  --text-primary: #1A1A1A;
  --text-secondary: #666666;
  --text-tertiary: #9E9E9E;
}

[data-theme="dark"] {
  /* Dark Mode */
  --glass-bg: rgba(40, 40, 42, 0.65);
  --glass-bg-hover: rgba(55, 55, 57, 0.75);
  --glass-border: rgba(255, 255, 255, 0.06);
  --glass-shadow: 0 20px 40px rgba(0, 0, 0, 0.25), 0 0 0 1px rgba(255, 255, 255, 0.03);

  --accent: #5EEAD4;
  --accent-light: #99F6E4;
  --accent-subtle: rgba(20, 184, 166, 0.2);
  --accent-glow: rgba(20, 184, 166, 0.2);
  --selection-bg: rgba(20, 184, 166, 0.2);
  --selection-border: rgba(20, 184, 166, 0.4);
  --highlight-bg: rgba(20, 184, 166, 0.3);
  --highlight-text: #5EEAD4;

  --text-primary: #FFFFFF;
  --text-secondary: #98989F;
  --text-tertiary: #636366;
}
```

---

## 字体系统

### 字体栈

```css
:root {
  --font-display: 'HarmonyOS Sans SC', 'PingFang SC', 'Microsoft YaHei', sans-serif;
  --font-body: 'HarmonyOS Sans SC', 'PingFang SC', 'Microsoft YaHei', sans-serif;
  --font-mono: 'SF Mono', 'Consolas', 'HarmonyOS Sans SC', monospace;
}
```

### 字体尺寸

| 样式 | 大小 | 字重 | 行高 | 用途 |
|------|------|------|------|------|
| Display | 28px | 600 | 1.2 | 大标题 |
| H1 | 20px | 600 | 1.3 | 页面标题 |
| H2 | 16px | 500 | 1.4 | 区块标题 |
| Body | 14px | 400 | 1.5 | 默认正文 |
| Caption | 12px | 400 | 1.4 | 辅助说明 |
| Small | 11px | 400 | 1.3 | 标签文字 |

---

## 间距系统

### 基础间距

```css
:root {
  --space-1: 4px;
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  --space-5: 20px;
  --space-6: 24px;
  --space-8: 32px;
}
```

### 组件间距

| 场景 | 值 | 说明 |
|------|---|------|
| 结果项内边距 | 12px 16px | 标准结果项 |
| 结果项间距 | 4px | 垂直列表 |
| 区块间距 | 8px | 应用和文件分组之间 |
| 窗口边距 | 24px | 窗口边缘 |

---

## 圆角系统

```css
:root {
  --radius-sm: 8px;    /* 小元素 */
  --radius-md: 12px;   /* 结果项 */
  --radius-lg: 16px;   /* 容器 */
  --radius-xl: 20px;   /* 大容器 */
  --radius-2xl: 28px;  /* 搜索栏 */
  --radius-full: 9999px; /* 圆形 */
}
```

---

## 阴影系统

```css
:root {
  /* 浅色模式阴影 */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.05);
  --shadow-md: 0 4px 6px rgba(0, 0, 0, 0.07);
  --shadow-lg: 0 8px 16px rgba(0, 0, 0, 0.1);
  
  /* 玻璃效果阴影 */
  --shadow-glass: 
    0 8px 32px rgba(0, 0, 0, 0.1),
    0 2px 8px rgba(0, 0, 0, 0.05);
  
  /* 搜索栏发光 */
  --shadow-glow: 0 0 60px var(--accent-glow);
}

[data-theme="dark"] {
  /* 深色模式阴影 */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.2);
  --shadow-md: 0 4px 6px rgba(0, 0, 0, 0.3);
  --shadow-lg: 0 8px 16px rgba(0, 0, 0, 0.4);
}
```

---

## 组件样式

### 搜索栏

```css
.search-bar {
  /* 布局 */
  width: 100%;
  height: 52px;
  padding: 0 6px;
  display: flex;
  align-items: center;
  gap: 10px;

  /* 视觉 */
  background: rgba(255, 255, 255, 0.7);
  border: 1px solid rgba(255, 255, 255, 0.6);
  border-radius: var(--radius-lg);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);

  /* 阴影 */
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.05), inset 0 1px 0 rgba(255, 255, 255, 0.8);

  /* 交互 */
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.search-bar:focus-within {
  background: rgba(255, 255, 255, 0.95);
  border-color: var(--selection-border);
  box-shadow: 0 4px 24px var(--accent-glow), inset 0 1px 0 rgba(255, 255, 255, 0.9);
}

.search-bar input {
  flex: 1;
  border: none;
  background: transparent;
  font-size: 16px;
  color: var(--text-primary);
  outline: none;
}

.search-bar input::placeholder {
  color: var(--text-tertiary);
}
```

### 结果项

```css
.result-item {
  /* 布局 */
  height: 56px;
  padding: 8px 16px;
  margin: 4px 12px;
  display: flex;
  align-items: center;
  gap: 12px;

  /* 视觉 */
  background: rgba(255, 255, 255, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.3);
  border-radius: var(--radius-md);

  /* 交互 */
  cursor: pointer;
  transition: all 180ms ease;
}

.result-item:hover {
  background: rgba(255, 255, 255, 0.8);
  transform: translateX(4px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
}

.result-item.selected {
  background: var(--selection-bg);
  border-color: var(--selection-border);
  box-shadow: 0 4px 16px var(--accent-glow);
}

.result-item.selected::before {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 24px;
  background: var(--accent);
  border-radius: 0 2px 2px 0;
}

[data-theme="dark"] .result-item {
  background: rgba(30, 30, 35, 0.4);
  border-color: rgba(255, 255, 255, 0.08);
}

[data-theme="dark"] .result-item:hover {
  background: rgba(50, 50, 60, 0.6);
}
```

### 分类标题

```css
.section-title {
  /* 字体 */
  font-size: 11px;
  font-weight: 500;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  
  /* 布局 */
  padding: 8px 16px;
  margin-bottom: 4px;
}
```

### 状态栏

```css
.status-bar {
  /* 布局 */
  height: 36px;
  padding: 0 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  
  /* 视觉 */
  background: rgba(0, 0, 0, 0.03);
  border-top: 1px solid var(--glass-border);
  
  /* 字体 */
  font-size: 11px;
  color: var(--text-tertiary);
}

[data-theme="dark"] .status-bar {
  background: rgba(255, 255, 255, 0.03);
}
```

### 快捷键标签

```css
.shortcut-key {
  /* 布局 */
  padding: 2px 6px;
  min-width: 20px;
  text-align: center;
  
  /* 视觉 */
  background: var(--gray-100);
  border: 1px solid var(--gray-200);
  border-radius: var(--radius-sm);
  border-bottom-width: 2px;
  
  /* 字体 */
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 500;
  color: var(--text-secondary);
}

[data-theme="dark"] .shortcut-key {
  background: var(--gray-800);
  border-color: var(--gray-700);
  color: var(--text-secondary);
}
```

---

## 动画规范

### 缓动函数

```css
:root {
  --ease-default: cubic-bezier(0.4, 0, 0.2, 1);
  --ease-in: cubic-bezier(0.4, 0, 1, 1);
  --ease-out: cubic-bezier(0, 0, 0.2, 1);
  --ease-spring: cubic-bezier(0.34, 1.56, 0.64, 1);
}
```

### 动画时长

| 类型 | 时长 | 用途 |
|------|------|------|
| Fast | 100ms | 悬停状态、微交互 |
| Normal | 150-200ms | 标准过渡 |
| Slow | 300ms | 主题切换 |

### 关键动画

```css
/* 窗口显示 */
@keyframes window-enter {
  from {
    opacity: 0;
    transform: scale(0.95) translateY(-10px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

/* 窗口隐藏 */
@keyframes window-exit {
  from {
    opacity: 1;
    transform: scale(1);
  }
  to {
    opacity: 0;
    transform: scale(0.95);
  }
}

/* 内容淡入 */
@keyframes fade-in {
  from {
    opacity: 0;
    transform: translateY(5px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 脉冲加载 */
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}
```

### 应用示例

```css
/* 窗口动画 */
.window-enter {
  animation: window-enter 200ms var(--ease-out) forwards;
}

.window-exit {
  animation: window-exit 150ms var(--ease-in) forwards;
}

/* 结果项交错动画 */
.result-item {
  opacity: 0;
  animation: fade-in 150ms var(--ease-out) forwards;
}

.result-item:nth-child(1) { animation-delay: 0ms; }
.result-item:nth-child(2) { animation-delay: 20ms; }
.result-item:nth-child(3) { animation-delay: 40ms; }
/* ... */

/* 骨架屏脉冲 */
.skeleton {
  animation: pulse 1.5s ease-in-out infinite;
}
```

---

## 布局规范

### 主窗口尺寸

| 状态 | 宽度 | 高度 | 说明 |
|------|------|------|------|
| 固定 | 720px | 520px | 搜索栏 + 结果列表，固定尺寸 |

### 内容区域

```
┌─────────────────────────────────────────────────────────────────┐
│  [拖拽区域 32px]                                                 │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │  🔍  搜索应用或文件...                    搜索 | 剪贴板 │   │
│   │                    搜索栏 52px                           │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │  常用应用                                               │   │
│   │  ┌──────────────────────────────────────────────────┐  │   │
│   │  │ ▪  🌐  Google Chrome                   应用  ↵  │  │   │
│   │  │ ▪  📝  Visual Studio Code              应用     │  │   │
│   │  │  ...                                            │  │   │
│   │  └──────────────────────────────────────────────────┘  │   │
│   │  文件与文件夹                                           │   │
│   │  ┌──────────────────────────────────────────────────┐  │   │
│   │  │ ▪  📁  Documents                       文件夹    │  │   │
│   │  │  ...                                            │  │   │
│   │  └──────────────────────────────────────────────────┘  │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│   ─────────────────────────────────────────────────────────      │
│   ⚙️ 设置  ↑↓ 选择  Enter 打开  Alt+数字 快速打开  Esc 关闭      │
└─────────────────────────────────────────────────────────────────┘
```

---

## 图标规范

### 尺寸

| 用途 | 尺寸 | 说明 |
|------|------|------|
| 搜索结果图标 | 20px | 应用/文件图标 |
| 搜索栏图标 | 20px | 搜索/清除图标 |
| 状态栏图标 | 14px | 提示图标 |

### 图标库

推荐使用：
- **TDesign Icons** - 官方图标库
- **Phosphor Icons** - 现代化线性图标

---

## 无障碍设计

### 对比度
- 文字与背景对比度至少 4.5:1
- 选中状态清晰可见

### 焦点状态
```css
:focus-visible {
  outline: 2px solid var(--primary-500);
  outline-offset: 2px;
}
```

### 减少动画
```css
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

---

**文档版本**: v2.2
**最后更新**: 2026-05-01
