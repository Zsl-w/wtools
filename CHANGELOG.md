# 变更日志

## 2026-05-12 — 修复 Enter 键第二次搜索后失效

### 问题
- 第一次搜索按 Enter 可以正常打开应用/文件
- 第二次搜索后按 Enter 无反应，无法打开选中项

### 根因分析
- 搜索框 `TextField` 持有焦点时，Flutter 单行文本框会消费 Enter 键事件（阻止换行），导致事件不冒泡到外层 `Focus.onKeyEvent` 处理器
- 整个 Enter 打开逻辑依赖外层 `Focus` 的按键冒泡，当 `TextField` 拥有焦点时此通路不可靠

### 修复
- **search_bar_widget.dart**: 新增 `onSubmitted` 回调参数，绑定到 `TextField.onSubmitted`
- **main_window_page.dart**: 传递 `_openSelectedSearch` 给 `SearchBarWidget.onSubmitted`，使 Enter 直接触发打开逻辑；新增 `_isOpening` 重入守卫防止重复触发

### 改动文件列表
- `lib/lib/src/widgets/search_bar_widget.dart` — 新增 `onSubmitted` 回调 + `TextField.onSubmitted` 绑定
- `lib/lib/src/pages/main_window_page.dart` — 传递回调 + 重入守卫
- `CHANGELOG.md` — 记录本次修复

## 2026-05-10 — 代码审查修复（安全、性能、架构）

### 修复项

#### 安全修复
- **ResultItem**: 修复 `v-html` 高亮偏移 bug——旧算法在 `escapeHtml` 后使用原始字符串索引切片，导致包含 `&<>"'` 的文件名高亮位置错误。改为先对全文转义，再在转义后文本中做匹配插入 `<mark>`
- **launch_app / open_file / show_in_folder**: 将 `cmd /C start` 替换为 `ShellExecuteW` Win32 API，避免 shell 解释注入风险

#### 性能优化
- **剪贴板图片加载**: 历史记录中的图片改为返回缩略图（~5-10KB），而非全尺寸原图（~1-2MB），切换剪贴板 Tab 时数据传输量降低 95%+
- **get_clipboard_history**: 返回类型从 `Vec<ClipboardItem>` 改为 `Result<Vec<ClipboardItem>, String>`，前端可感知错误
- **scan_shortcuts**: 添加 `depth` 递归深度参数（开始菜单/桌面限 2-3 层），防止深层嵌套目录扫描耗时过长

#### 架构改进
- **剪贴板监视线程**: `stop_clipboard_monitor` 现在使用 `JoinHandle` 等待线程退出，而非仅设标志位，避免进程退出时资源泄漏
- **主题持久化**: `settings.ts` 的 `theme` 从 Pinia 内存态改为 `localStorage` 持久存储，刷新后不丢失
- **删除死代码**: 移除 `preview_handler` 模块（`com_interfaces.rs`、`host.rs`、`mod.rs`），该模块实现了 Windows Shell Preview Handler 但从未被任何 Tauri command 调用
- **删除未使用组件**: 移除 `StatusBar/index.vue`（从未被引用）
- **日志统一**: 全面引入 `log` + `env_logger` crate，所有 `println!` → `log::info!`，`eprintln!` → `log::error!`/`log::warn!`；`everything/log.rs` 的文件日志改为转发到 `log` crate

### 改动文件列表
- `src/components/ResultItem/index.vue` — 高亮算法重写
- `src/components/ClipboardList/index.vue` — 复制图片改用 `get_clipboard_image` 命令
- `src/components/StatusBar/` — 删除整个目录
- `src/stores/settings.ts` — 主题 localStorage 持久化
- `src-tauri/Cargo.toml` — 添加 `log`、`env_logger` 依赖
- `src-tauri/src/lib.rs` — 移除 `preview_handler` 模块，注册 `get_clipboard_image` 命令，初始化 logger，替换 println/eprintln
- `src-tauri/src/commands/app.rs` — `launch_app`/`open_file`/`show_in_folder` 改用 `ShellExecuteW`，`scan_shortcuts` 添加深度限制
- `src-tauri/src/commands/clipboard.rs` — `get_clipboard_history` 返回缩略图+Result 类型，新增 `get_clipboard_image` 命令
- `src-tauri/src/commands/file.rs` — 替换 eprintln
- `src-tauri/src/clipboard/history.rs` — `add_image` 生成缩略图，`remove_item`/`clear`/`cleanup_images` 处理缩略图文件，新增 `image_dir()` 方法
- `src-tauri/src/clipboard/monitor.rs` — `JoinHandle` 优雅退出，替换 println/eprintln
- `src-tauri/src/everything/log.rs` — 简化为转发 `log` crate
- `src-tauri/src/preview_handler/` — 删除整个目录

## 2026-05-09 — Flutter 窗口渲染与键盘导航修复

### 问题
- 窗口半透明背景 + BackdropFilter 导致渲染效果极差，视觉模糊不清
- 窗口顶部四分之一区域显示为黑色，内容被挤压变形
- 搜索输入框中退格键（Backspace/Delete）无反应
- 上下键切换选中项时出现两个灰色背景重叠的"跳动"动画
- 列表不会跟随选中项自动滚动
- 标题（"常用应用"）会随列表滚走被遮挡
- 循环到最后一个再按向下键，视角不会回到第一个

### 改动

#### 窗口渲染修复
- **main.dart**: 窗口背景色 `0xAA1A1A1A` → `0xFF1C1C1E`（全不透明）
- **glass_container.dart**: 移除 `BackdropFilter`，在不透明表面上无意义且消耗 GPU；渐变颜色 `0xEE` → `0xFF`

#### 布局修复
- **main_window_page.dart**: `Scaffold` 背景 `Colors.transparent` → 不透明 `0xFF1C1C1E`；移除 `Center` 包裹层，`GlassContainer` 改为 `double.infinity` 铺满窗口

#### 键盘事件修复
- **main_window_page.dart**: `onKeyEvent` 始终返回 `KeyEventResult.handled` → 改为只拦截 Tab/Esc/方向键/回车，其余透传给 `TextField`

#### 选中与滚动
- **result_item_widget.dart**: `AnimatedContainer`（150ms 动画）→ `Container`（瞬间切换），消除两个灰色背景重叠
- **result_list_widget.dart**: `ConsumerWidget` → `ConsumerStatefulWidget`，新增 `ScrollController`；标题（"常用应用"）移出 `ListView` 固定在上方不参与滚动；改用 `ScrollController.animateTo` 计算偏移按需滚动

### 影响范围
- 窗口现在正常显示，无黑色区域、无内容挤压
- 退格键可正常删除输入
- 选中切换即时无动画残留
- 列表仅在选中项超出可视区域时滚动
- 循环到头时直接回到顶部

## 2026-05-09 — 应用行 + 文件列表双区域搜索布局

### 问题
- 应用和文件混在同一个 ListView 中，用统一上下键导航
- 文件预览/打开逻辑依赖全局 `selectedIndex`，逻辑耦合度高
- 应用项和文件项格式不同但共用同一渲染路径

### 改动

#### 状态模型重构（`search_provider.dart`）
- 新增 `FocusArea` 枚举（`appRow` / `fileList`）
- `SearchState`：删除 `results` 和 `selectedIndex`；新增 `appResults`、`fileResults`、`selectedAppIndex`、`selectedFileIndex`、`focusArea`
- `SearchNotifier`：删除 `selectNext()` / `selectPrev()`；新增 `selectNextApp()`、`selectPrevApp()`、`selectNextFile()`、`selectPrevFile()`、`moveFocusToFileList()`、`moveFocusToAppRow()`
- `_executeSearch` 分阶段分别填充 `appResults` 和 `fileResults`
- 新搜索时重置 `selectedAppIndex=0`、`focusArea=appRow`
- 无应用结果时自动切 `focusArea` 到 `fileList`

#### 新增应用行组件（`app_row_widget.dart`）
- 新建文件，横向 `ListView.builder`，每个卡片 72×62px
- 卡片内容：28px 图标 + 10.5px 名称单行截断
- 选中态：accent 半透明背景 + accent 边框 + 左侧指示条
- 自带水平 `ScrollController`，选中超出可视区时自动滚动
- 卡片点击直接启动应用

#### 结果列表重构（`result_list_widget.dart`）
- 移除应用项渲染（移到 `AppRowWidget`）
- 顶部固定：section header + AppRowWidget
- 下方 Expanded：纯文件 ListView
- `_scrollToSelected` 改为仅处理文件选中滚动

#### 键盘导航重写（`main_window_page.dart`）
- 左右键 → 应用行内导航
- 下键 → 应用行切换到文件列表顶部
- 上键 → 文件列表顶部切换回应用行
- `_openSelectedSearch` 根据 `focusArea` 取对应索引
- `_showPreview` 仅在 `focusArea==fileList` 时显示
- 底部提示更新为 `←→ 应用` + `↑↓ 文件`

### 影响范围
- 搜索区域分为上下两层：应用横向卡片行 + 文件竖向列表
- 键盘导航区域感知，操作更符合直觉
- 文件预览仅在文件列表聚焦时显示，体验更干净
