# wTools 项目规范

## 语言
- 所有回复和交流必须使用中文
- 代码注释使用英文（保持惯例）

## 变更日志
- 每次代码改动都必须同步更新 `CHANGELOG.md`
- 记录格式：日期、改动文件列表、改动内容描述、原因/目的
- 在每次提交或完成一组相关改动后更新

## 项目状态
- 当前正在进行 Tauri v2 + Vue 3 → Flutter + Rust + flutter_rust_bridge 迁移
- Flutter 项目位于 `lib/` 目录
- Rust 后端代码位于 `lib/rust/src/`
- 不要改动 `src/` 和 `src-tauri/`（旧版 Tauri 代码，迁移完成后删除）
