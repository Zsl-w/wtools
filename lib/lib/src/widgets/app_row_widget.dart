import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:window_manager/window_manager.dart';
import '../models/search_result.dart';
import '../providers/search_provider.dart';
import '../rust/api/app.dart' as rust_app;
import '../theme/app_theme.dart';
import '../utils/icon_cache.dart';

/// Horizontal scrollable row of app cards.
///
/// Reads [searchProvider] to display [SearchState.appResults] and
/// highlight the card at [SearchState.selectedAppIndex] when
/// [SearchState.focusArea] is [FocusArea.appRow].
class AppRowWidget extends ConsumerStatefulWidget {
  const AppRowWidget({super.key});

  @override
  ConsumerState<AppRowWidget> createState() => _AppRowWidgetState();
}

class _AppRowWidgetState extends ConsumerState<AppRowWidget> {
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
      const cardWidth = 72.0;
      const gap = 8.0;
      final itemLeft = index * (cardWidth + gap);
      final viewportWidth = _scrollController.position.viewportDimension;
      final itemRight = itemLeft + cardWidth;
      final offset = _scrollController.offset;

      if (itemLeft < offset) {
        _scrollController.animateTo(
          itemLeft.clamp(0.0, _scrollController.position.maxScrollExtent),
          duration: const Duration(milliseconds: 60),
          curve: Curves.easeOut,
        );
      } else if (itemRight > offset + viewportWidth) {
        _scrollController.animateTo(
          (itemRight - viewportWidth + 8)
              .clamp(0.0, _scrollController.position.maxScrollExtent),
          duration: const Duration(milliseconds: 60),
          curve: Curves.easeOut,
        );
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final apps = ref.watch(searchProvider.select((s) => s.appResults));
    final selectedIndex =
        ref.watch(searchProvider.select((s) => s.selectedAppIndex));
    final focusArea =
        ref.watch(searchProvider.select((s) => s.focusArea));

    if (apps.isEmpty) return const SizedBox.shrink();

    _scrollToSelected(selectedIndex);

    return RepaintBoundary(
      child: SizedBox(
        height: 78,
        child: ListView.builder(
          controller: _scrollController,
          scrollDirection: Axis.horizontal,
          padding: const EdgeInsets.fromLTRB(24, 8, 12, 8),
          itemCount: apps.length,
          itemBuilder: (context, index) {
            return _AppCard(
              key: ValueKey(apps[index].id),
              item: apps[index],
              isSelected: focusArea == FocusArea.appRow &&
                  selectedIndex == index,
            );
          },
        ),
      ),
    );
  }
}

/// A single app card in the horizontal row.
class _AppCard extends StatefulWidget {
  final SearchResult item;
  final bool isSelected;

  const _AppCard({super.key, required this.item, required this.isSelected});

  @override
  State<_AppCard> createState() => _AppCardState();
}

class _AppCardState extends State<_AppCard> {
  Uint8List? _iconBytes;

  @override
  void initState() {
    super.initState();
    _loadIcon();
  }

  @override
  void didUpdateWidget(_AppCard oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.item.path != widget.item.path) {
      _iconBytes = null;
    }
    // Re-check cache on any rebuild (e.g. after preload completes)
    if (_iconBytes == null) {
      final cached = IconCache.get(widget.item.path);
      if (cached != null) {
        _iconBytes = cached;
        return;
      }
      _loadIcon();
    }
  }

  Future<void> _loadIcon() async {
    final cached = IconCache.get(widget.item.path);
    if (cached != null) {
      if (mounted) setState(() { _iconBytes = cached; });
      return;
    }
    final bytes = await IconCache.loadAppIcon(widget.item.path);
    if (bytes != null && mounted) setState(() { _iconBytes = bytes; });
  }

  void _open() async {
    try {
      await rust_app.launchApp(path: widget.item.path);
      await windowManager.hide();
      await windowManager.blur();
    } catch (_) {}
  }

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: _open,
      child: Container(
        width: 72,
        height: 62,
        margin: const EdgeInsets.only(right: 8),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(10),
          color: widget.isSelected
              ? const Color(0xFFE8E8E8)
              : Colors.transparent,
          border: widget.isSelected
              ? Border.all(
                  color: AppColors.accent.withValues(alpha: 0.35),
                  width: 0.5,
                )
              : null,
        ),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            // Icon (28×28)
            SizedBox(
              width: 28,
              height: 28,
              child: _buildIcon(),
            ),
            const SizedBox(height: 4),
            // Name (single line)
            Text(
              widget.item.name,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              textAlign: TextAlign.center,
              style: const TextStyle(
                fontSize: 10.5,
                fontWeight: FontWeight.w500,
                color: AppColors.textPrimary,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildIcon() {
    if (_iconBytes != null && _iconBytes!.isNotEmpty) {
      return ClipRRect(
        borderRadius: BorderRadius.circular(5),
        child: Image.memory(
          _iconBytes!,
          width: 28,
          height: 28,
          fit: BoxFit.cover,
          errorBuilder: (_, _, _) => _fallbackIcon(),
        ),
      );
    }
    return _fallbackIcon();
  }

  Widget _fallbackIcon() {
    return Container(
      width: 28,
      height: 28,
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(5),
        color: AppColors.typeApp.withValues(alpha: 0.12),
      ),
      child: const Icon(Icons.apps_rounded, size: 16, color: AppColors.typeApp),
    );
  }
}
