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

## Skill 调度规则（最高优先级）

每次收到用户请求，必须先按下方映射表检查是否需要加载 skill。**匹配即加载，不得跳过。**

| 用户需求关键词 | 加载的 skill |
|---------------|-------------|
| "设计/开发/实现/添加" + 功能 | `brainstorming` → `writing-plans` → `test-driven-development` |
| "修复/调试/bug/错误/不工作/异常" | `systematic-debugging` |
| "审查/review/检查代码" | `requesting-code-review` |
| "设计页面/UI/界面/样式/美化/landing/dashboard" | `frontend-design` |
| "PPT/演示文稿/幻灯片" | `pptx` |
| "PDF/导出PDF" | `pdf` |
| "Excel/表格/xlsx/csv" | `xlsx` |
| "Word/文档/docx/报告" | `docx` |
| "画图/流程图/可视化/diagram/思维导图" | `mermaid-visualizer` 或 `excalidraw-diagram` |
| "提交/commit/PR/合并" | `finishing-a-development-branch` |
| "学习/quiz/测验" | `tutor` |
| "打开网页/浏览器/截图" | `agent-browser` |
| "生成图片/AI绘画" | `ai-image-generation` |

**铁律**：
- 写任何代码前必须先 `brainstorming` → 确认设计 → `writing-plans` → 再实施
- 声称"修好了"前必须先 `verification-before-completion` 运行验证
- 1% 的可能匹配就要加载，100% 匹配必须加载
