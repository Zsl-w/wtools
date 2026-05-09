import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../providers/search_provider.dart';
import '../rust/api/app.dart' as rust_app;
import '../theme/app_theme.dart';
import 'app_row_widget.dart';
import 'result_item_widget.dart';

/// Result list — a fixed app row at the top, then a scrollable file list below.
class ResultListWidget extends ConsumerStatefulWidget {
  const ResultListWidget({super.key});

  @override
  ConsumerState<ResultListWidget> createState() => _ResultListWidgetState();
}

class _ResultListWidgetState extends ConsumerState<ResultListWidget> {
  final ScrollController _scrollController = ScrollController();

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  void _scrollToSelected(int index) {
    if (index < 0) return;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!_scrollController.hasClients) return;

      final state = ref.read(searchProvider);
      if (index >= state.fileResults.length) return;

      final itemTop = index * 60.0;
      final viewportStart = _scrollController.offset;
      final viewportEnd =
          viewportStart + _scrollController.position.viewportDimension;
      final itemBottom = itemTop + 56;

      if (itemTop < viewportStart) {
        _scrollController.animateTo(
          itemTop.clamp(0.0, _scrollController.position.maxScrollExtent),
          duration: const Duration(milliseconds: 60),
          curve: Curves.easeOut,
        );
      } else if (itemBottom > viewportEnd) {
        final extra = itemBottom - viewportEnd + 8;
        _scrollController.animateTo(
          (_scrollController.offset + extra)
              .clamp(0.0, _scrollController.position.maxScrollExtent),
          duration: const Duration(milliseconds: 60),
          curve: Curves.easeOut,
        );
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(searchProvider);
    final query = state.query;
    final hasQuery = query.isNotEmpty;

    if (state.focusArea == FocusArea.fileList) {
      _scrollToSelected(state.selectedFileIndex);
    }

    if (state.error != null &&
        state.appResults.isEmpty &&
        state.fileResults.isEmpty) {
      return _buildError(context, state.error!);
    }

    if (state.appResults.isEmpty &&
        state.fileResults.isEmpty &&
        !state.isLoading) {
      return _buildEmpty(context);
    }

    return Column(
      children: [
        // ── App row (fixed, never scrolls) ──
        if (state.appResults.isNotEmpty) ...[
          _buildSectionHeader(
            context,
            hasQuery ? '应用程序' : '常用应用',
          ),
          const RepaintBoundary(child: AppRowWidget()),
        ],
        // ── File section ──
        if (state.fileResults.isNotEmpty && hasQuery)
          _buildSectionHeader(context, '文件'),
        if (state.fileResults.isNotEmpty && hasQuery)
          Expanded(
            child: RepaintBoundary(
              child: ListView(
                controller: _scrollController,
                padding: const EdgeInsets.only(top: 4, bottom: 8),
                children: [
                  ...state.fileResults.asMap().entries.map((e) {
                    return ResultItemWidget(
                      key: ValueKey(e.value.id),
                      item: e.value,
                      index: e.key,
                      isSelected: state.focusArea == FocusArea.fileList &&
                          state.selectedFileIndex == e.key,
                      query: query,
                    );
                  }),
                  if (state.error != null)
                    _buildErrorBanner(context, state.error!),
                  if (state.isLoading)
                    const Padding(
                      padding: EdgeInsets.all(24),
                      child: Center(
                        child: SizedBox(
                          width: 20,
                          height: 20,
                          child: CircularProgressIndicator(
                            strokeWidth: 2,
                            valueColor:
                                AlwaysStoppedAnimation(AppColors.accent),
                          ),
                        ),
                      ),
                    ),
                ],
              ),
            ),
          )
        else if (state.appResults.isNotEmpty &&
            !hasQuery &&
            !state.isLoading)
          const Spacer(),
        if (hasQuery)
          _buildAddCustomApp(context, query, ref),
        if (state.isLoading && state.fileResults.isEmpty)
          const Padding(
            padding: EdgeInsets.all(24),
            child: Center(
              child: SizedBox(
                width: 20,
                height: 20,
                child: CircularProgressIndicator(
                  strokeWidth: 2,
                  valueColor: AlwaysStoppedAnimation(AppColors.accent),
                ),
              ),
            ),
          ),
        const SizedBox(height: 4),
      ],
    );
  }

  Widget _buildSectionHeader(BuildContext context, String title) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    return Padding(
      padding: const EdgeInsets.fromLTRB(24, 8, 24, 4),
      child: Row(
        children: [
          Text(
            title,
            style: TextStyle(
              fontSize: 11.5,
              fontWeight: FontWeight.w600,
              letterSpacing: 0.5,
              color: isDark
                  ? AppColors.textTertiaryDark
                  : AppColors.textTertiaryLight,
            ),
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Divider(
              height: 1,
              color: Colors.black.withValues(alpha: isDark ? 0.08 : 0.05),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildEmpty(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(48),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.search_off_rounded,
              size: 40,
              color: isDark
                  ? AppColors.textTertiaryDark.withValues(alpha: 0.4)
                  : AppColors.textTertiaryLight.withValues(alpha: 0.4),
            ),
            const SizedBox(height: 12),
            Text(
              '输入关键词搜索应用和文件',
              style: TextStyle(
                fontSize: 13,
                color: isDark
                    ? AppColors.textTertiaryDark
                    : AppColors.textTertiaryLight,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildError(BuildContext context, String error) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(48),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.error_outline_rounded,
                size: 40, color: Colors.redAccent),
            const SizedBox(height: 12),
            Text(error,
                textAlign: TextAlign.center,
                style:
                    const TextStyle(fontSize: 13, color: Colors.redAccent)),
          ],
        ),
      ),
    );
  }

  Widget _buildErrorBanner(BuildContext context, String error) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    return Container(
      margin: const EdgeInsets.symmetric(horizontal: 24, vertical: 8),
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(10),
        color: isDark
            ? const Color(0x33FF6B6B)
            : const Color(0x1AFF6B6B),
        border: Border.all(
          color: Colors.redAccent.withValues(alpha: 0.2),
        ),
      ),
      child: Row(
        children: [
          const Icon(Icons.warning_amber_rounded,
              size: 16, color: Colors.redAccent),
          const SizedBox(width: 8),
          Expanded(
            child: Text(error,
                style: const TextStyle(
                    fontSize: 11.5, color: Colors.redAccent)),
          ),
        ],
      ),
    );
  }

  Widget _buildAddCustomApp(
      BuildContext context, String query, WidgetRef ref) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 8),
      child: GestureDetector(
        onTap: () async {
          final result = await FilePicker.platform.pickFiles(
            type: FileType.custom,
            allowedExtensions: ['exe'],
          );
          if (result != null && result.files.isNotEmpty) {
            final path = result.files.first.path!;
            final name = path.split('\\').last.replaceAll('.exe', '');
            await rust_app.addCustomApp(name: name, path: path);
            await ref.read(searchProvider.notifier).refreshIndex();
            ref.read(searchProvider.notifier).search(query);
          }
        },
        child: Container(
          height: 44,
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: AppColors.accent.withValues(alpha: 0.3),
              style: BorderStyle.solid,
            ),
          ),
          child: Center(
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Icon(Icons.add_rounded,
                    size: 18, color: AppColors.accent),
                const SizedBox(width: 6),
                const Text(
                  '添加自定义应用',
                  style: TextStyle(
                    fontSize: 13,
                    fontWeight: FontWeight.w500,
                    color: AppColors.accent,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
