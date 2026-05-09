# 变更日志

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
