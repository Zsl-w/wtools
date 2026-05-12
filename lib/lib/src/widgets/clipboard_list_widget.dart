import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:window_manager/window_manager.dart';
import '../providers/clipboard_provider.dart';
import '../rust/clipboard_impl/history.dart';
import '../theme/app_theme.dart';
import '../utils/time_format.dart';

class ClipboardListWidget extends ConsumerStatefulWidget {
  const ClipboardListWidget({super.key});

  @override
  ConsumerState<ClipboardListWidget> createState() => _ClipboardListWidgetState();
}

class _ClipboardListWidgetState extends ConsumerState<ClipboardListWidget> {
  final ScrollController _scrollController = ScrollController();
  bool _showToast = false;

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  void _scrollToSelected(int index) {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!_scrollController.hasClients) return;
      const itemExtent = 100.0;
      const topPad = 4.0;
      final itemTop = topPad + index * itemExtent;
      final viewportHeight = _scrollController.position.viewportDimension;
      final offset = _scrollController.offset;

      if (itemTop - 2 < offset) {
        _scrollController.animateTo(
          (itemTop - 2).clamp(0.0, _scrollController.position.maxScrollExtent),
          duration: const Duration(milliseconds: 80),
          curve: Curves.easeOut,
        );
      } else if (itemTop + itemExtent + 2 > offset + viewportHeight) {
        _scrollController.animateTo(
          (itemTop - viewportHeight + itemExtent + 2)
              .clamp(0.0, _scrollController.position.maxScrollExtent),
          duration: const Duration(milliseconds: 80),
          curve: Curves.easeOut,
        );
      }
    });
  }

  Future<void> _copyAndHide() async {
    final success = await ref.read(clipboardProvider.notifier).copySelected();
    if (success && mounted) {
      setState(() => _showToast = true);
      Future.delayed(const Duration(milliseconds: 400), () async {
        if (mounted) {
          setState(() => _showToast = false);
          await windowManager.hide();
          await windowManager.blur();
        }
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(clipboardProvider);
    final items = state.filteredItems;

    if (state.items.isEmpty) {
      return _buildEmpty(context);
    }

    if (state.query.isNotEmpty && items.isEmpty) {
      return _buildNoResults(context);
    }

    _scrollToSelected(state.selectedIndex);

    return Stack(
      children: [
        Column(
          children: [
            // Header
            _buildHeader(context, items.length, state.items.length),
            // List
            Expanded(
              child: ListView.builder(
                controller: _scrollController,
                padding: const EdgeInsets.only(top: 4, bottom: 40),
                itemExtent: 100,
                itemCount: items.length,
                itemBuilder: (context, index) {
                  final item = items[index];
                  final isSelected = state.selectedIndex == index;
                  return _buildItem(context, item, index, isSelected);
                },
              ),
            ),
          ],
        ),
        // Toast
        if (_showToast)
          Positioned(
            bottom: 16,
            left: 0,
            right: 0,
            child: Center(
              child: AnimatedOpacity(
                opacity: _showToast ? 1.0 : 0.0,
                duration: const Duration(milliseconds: 120),
                child: Container(
                  padding:
                      const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                  decoration: BoxDecoration(
                    borderRadius: BorderRadius.circular(20),
                    color: AppColors.primaryLight.withValues(alpha: 0.9),
                    boxShadow: [
                      BoxShadow(
                        color: AppColors.primaryLight.withValues(alpha: 0.3),
                        blurRadius: 12,
                      ),
                    ],
                  ),
                  child: const Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(Icons.check_rounded, size: 16, color: Colors.white),
                      SizedBox(width: 6),
                      Text('复制成功',
                          style: TextStyle(
                            fontSize: 13,
                            fontWeight: FontWeight.w600,
                            color: Colors.white,
                          )),
                    ],
                  ),
                ),
              ),
            ),
          ),
      ],
    );
  }

  Widget _buildHeader(BuildContext context, int shown, int total) {
    final title = shown == total
        ? '剪贴板历史 ($total)'
        : '剪贴板 ($shown / $total)';
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 10),
      child: Row(
        children: [
          Text(
            title,
            style: const TextStyle(
              fontSize: 11.5,
              fontWeight: FontWeight.w600,
              letterSpacing: 0.5,
              color: AppColors.textTertiary,
            ),
          ),
          const Spacer(),
          GestureDetector(
            onTap: () => ref.read(clipboardProvider.notifier).clearAll(),
            child: Text(
              '清空',
              style: TextStyle(
                fontSize: 11,
                fontWeight: FontWeight.w500,
                color: Colors.redAccent.withValues(alpha: 0.7),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildItem(BuildContext context, ClipboardItem item,
      int index, bool isSelected) {
    return GestureDetector(
      onTap: () {
        ref.read(clipboardProvider.notifier).selectIndex(index);
        _copyAndHide();
      },
      child: Container(
        margin: const EdgeInsets.symmetric(horizontal: 12, vertical: 2),
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(12),
          color: isSelected
              ? const Color(0xFFF0F0F0)
              : (item.pinned
                  ? AppColors.secondary.withValues(alpha: 0.04)
                  : Colors.transparent),
          border: isSelected
              ? Border.all(color: AppColors.accent.withValues(alpha: 0.35), width: 0.5)
              : null,
        ),
        child: Row(
          children: [
            // Selection indicator bar — thin left border
            Container(
              width: 10,
              height: 28,
              decoration: isSelected
                  ? BoxDecoration(
                      border: Border(
                        left: BorderSide(
                          color: AppColors.accent,
                          width: 1.5,
                        ),
                      ),
                    )
                  : null,
            ),
            // Content preview
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  if (item.contentType == 'image' &&
                      item.content != null &&
                      item.content!.contains(','))
                    ClipRRect(
                      borderRadius: BorderRadius.circular(4),
                      child: Image.memory(
                        base64Decode(item.content!.split(',').last),
                        height: 48,
                        fit: BoxFit.cover,
                        errorBuilder: (_, _, _) => const SizedBox.shrink(),
                      ),
                    )
                  else
                    Text(
                      item.preview,
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                      style: const TextStyle(
                        fontSize: 13,
                        color: AppColors.textPrimary,
                      ),
                    ),
                  const SizedBox(height: 3),
                  Row(
                    children: [
                      if (item.pinned)
                        Container(
                          padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                          margin: const EdgeInsets.only(right: 6),
                          decoration: BoxDecoration(
                            borderRadius: BorderRadius.circular(3),
                            color: AppColors.secondary.withValues(alpha: 0.15),
                          ),
                          child: const Text('📌',
                              style: TextStyle(fontSize: 9)),
                        ),
                      Text(
                        formatRelativeTime(item.timestamp.toInt()),
                        style: const TextStyle(
                          fontSize: 10.5,
                          color: AppColors.textTertiary,
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
            // Action buttons — outside Expanded, fixed position
            _actionButton(
              Icons.push_pin,
              item.pinned ? AppColors.secondary : null,
              () => ref.read(clipboardProvider.notifier).togglePin(item.id),
            ),
            const SizedBox(width: 4),
            _actionButton(
              Icons.close_rounded,
              Colors.redAccent,
              () => ref.read(clipboardProvider.notifier).deleteItem(item.id),
            ),
            const SizedBox(width: 6),
            // Open hint — always reserve 20px to prevent layout shift
            SizedBox(
              width: 20,
              height: 20,
              child: isSelected
                  ? Container(
                      decoration: BoxDecoration(
                        borderRadius: BorderRadius.circular(5),
                        color: AppColors.accent.withValues(alpha: 0.15),
                      ),
                      child: const Center(
                        child: Text(
                          '↵',
                          style: TextStyle(
                            fontSize: 11,
                            fontWeight: FontWeight.w700,
                            color: AppColors.accent,
                          ),
                        ),
                      ),
                    )
                  : const SizedBox.shrink(),
            ),
          ],
        ),
      ),
    );
  }

  Widget _actionButton(IconData icon, Color? color, VoidCallback onTap) {
    return GestureDetector(
      onTap: onTap,
      child: Icon(
        icon,
        size: 15,
        color: color?.withValues(alpha: 0.6) ?? Colors.grey.withValues(alpha: 0.5),
      ),
    );
  }

  Widget _buildNoResults(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.search_off_rounded, size: 40,
              color: AppColors.textTertiary.withValues(alpha: 0.3)),
          const SizedBox(height: 12),
          const Text('剪贴板中无匹配内容',
              style: TextStyle(fontSize: 13,
                  color: AppColors.textTertiary)),
        ],
      ),
    );
  }

  Widget _buildEmpty(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(
            Icons.content_paste_rounded,
            size: 40,
            color: AppColors.textTertiary.withValues(alpha: 0.3),
          ),
          const SizedBox(height: 12),
          const Text(
            '剪贴板为空',
            style: TextStyle(
              fontSize: 14,
              fontWeight: FontWeight.w500,
              color: AppColors.textTertiary,
            ),
          ),
          const SizedBox(height: 4),
          Text(
            '复制文本或图片后将自动记录',
            style: TextStyle(
              fontSize: 12,
              color: AppColors.textTertiary.withValues(alpha: 0.6),
            ),
          ),
        ],
      ),
    );
  }
}
