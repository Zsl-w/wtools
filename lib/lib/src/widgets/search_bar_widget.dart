import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../providers/search_provider.dart';
import '../providers/clipboard_provider.dart';
import '../theme/app_theme.dart';

class SearchBarWidget extends ConsumerStatefulWidget {
  final VoidCallback? onSubmitted;

  const SearchBarWidget({super.key, this.onSubmitted});

  @override
  ConsumerState<SearchBarWidget> createState() => SearchBarWidgetState();
}

class SearchBarWidgetState extends ConsumerState<SearchBarWidget> {
  final _controller = TextEditingController();
  final _focusNode = FocusNode();

  @override
  void dispose() {
    _controller.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  void focusInput() => _focusNode.requestFocus();

  void reset() {
    _controller.clear();
    ref.read(searchProvider.notifier).clearSearch();
    ref.read(clipboardProvider.notifier).search('');
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: _focusNode,
      builder: (context, _) {
        final focused = _focusNode.hasFocus;
        return Container(
          height: 52,
          padding: const EdgeInsets.symmetric(horizontal: 6),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(16),
            color: Colors.white.withValues(alpha: 0.72),
            border: Border.all(
              color: focused
                  ? AppColors.accent.withValues(alpha: 0.5)
                  : Colors.white.withValues(alpha: 0.5),
              width: focused ? 1.2 : 0.5,
            ),
            boxShadow: focused
                ? [
                    BoxShadow(
                      color: AppColors.accent.withValues(alpha: 0.2),
                      blurRadius: 16,
                      spreadRadius: -2,
                    ),
                  ]
                : [
                    BoxShadow(
                      color: Colors.black.withValues(alpha: 0.04),
                      blurRadius: 8,
                      offset: const Offset(0, 2),
                    ),
                  ],
          ),
          child: Row(
            children: [
              Container(
                width: 36,
                height: 36,
                decoration: BoxDecoration(
                  borderRadius: BorderRadius.circular(10),
                  color: AppColors.accent.withValues(alpha: 0.12),
                ),
                child: const Icon(
                  Icons.search_rounded,
                  size: 20,
                  color: AppColors.accent,
                ),
              ),
              const SizedBox(width: 10),
              Expanded(
                child: TextField(
                  controller: _controller,
                  focusNode: _focusNode,
                  onSubmitted: (_) => widget.onSubmitted?.call(),
                  onChanged: (v) {
                    final q = v.trim();
                    ref.read(searchProvider.notifier).search(q);
                    ref.read(clipboardProvider.notifier).search(q);
                  },
                  style: const TextStyle(
                    fontSize: 15,
                    fontWeight: FontWeight.w500,
                    color: AppColors.textPrimary,
                  ),
                  decoration: InputDecoration(
                    hintText: '搜索应用或文件...',
                    hintStyle: const TextStyle(
                      color: AppColors.textTertiary,
                      fontSize: 15,
                      fontWeight: FontWeight.w400,
                    ),
                    border: InputBorder.none,
                    contentPadding: EdgeInsets.zero,
                  ),
                ),
              ),
              if (_controller.text.isNotEmpty)
                GestureDetector(
                  onTap: reset,
                  child: Container(
                    width: 28,
                    height: 28,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: Colors.black.withValues(alpha: 0.06),
                    ),
                    child: const Icon(
                      Icons.close_rounded,
                      size: 16,
                      color: AppColors.textSecondary,
                    ),
                  ),
                ),
            ],
          ),
        );
      },
    );
  }
}
