import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:window_manager/window_manager.dart';
import '../models/search_result.dart';
import '../rust/api/app.dart' as rust_app;
import '../theme/app_theme.dart';
import '../utils/icon_cache.dart';

class ResultItemWidget extends StatefulWidget {
  final SearchResult item;
  final int index;
  final bool isSelected;
  final String query;

  const ResultItemWidget({
    super.key,
    required this.item,
    required this.index,
    required this.isSelected,
    required this.query,
  });

  @override
  State<ResultItemWidget> createState() => _ResultItemWidgetState();
}

class _ResultItemWidgetState extends State<ResultItemWidget> {
  Uint8List? _iconBytes;

  @override
  void initState() {
    super.initState();
    _loadIcon();
  }

  @override
  void didUpdateWidget(ResultItemWidget oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.item.path != widget.item.path) {
      _iconBytes = IconCache.get(widget.item.path);
      if (_iconBytes == null) _loadIcon();
    }
  }

  Future<void> _loadIcon() async {
    final cached = IconCache.get(widget.item.path);
    if (cached != null) {
      if (mounted) setState(() { _iconBytes = cached; });
      return;
    }
    Uint8List? bytes;
    if (widget.item.type == 'app') {
      bytes = await IconCache.loadAppIcon(widget.item.path);
    } else if (widget.item.extension != null) {
      const imageExts = ['png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'svg', 'ico'];
      if (imageExts.contains(widget.item.extension)) {
        bytes = await IconCache.loadFileThumb(widget.item.path);
      }
    }
    if (bytes != null && mounted) setState(() { _iconBytes = bytes; });
  }

  String _emojiForType() {
    switch (widget.item.type) {
      case 'app': return '';
      case 'folder': return '📁';
      default:
        if (widget.item.extension == null) return '📄';
        const emojiMap = {
          'pdf': '📕', 'doc': '📘', 'docx': '📘',
          'xls': '📊', 'xlsx': '📊', 'ppt': '📽',
          'pptx': '📽', 'png': '🖼', 'jpg': '🖼',
          'jpeg': '🖼', 'gif': '🖼',
          'mp3': '🎵', 'wav': '🎵', 'mp4': '🎬',
          'zip': '📦', 'rar': '📦', '7z': '📦',
          'txt': '📝', 'md': '📝',
          'js': '💛', 'ts': '💙', 'py': '🐍',
          'rs': '🦀', 'go': '🔵', 'dart': '🎯',
        };
        return emojiMap[widget.item.extension] ?? '📄';
    }
  }

  void _open() async {
    try {
      if (widget.item.type == 'app') {
        await rust_app.launchApp(path: widget.item.path);
      } else {
        await rust_app.openFile(path: widget.item.path);
      }
      // Hide window after opening
      await windowManager.hide();
      await windowManager.blur();
    } catch (_) {}
  }

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final color = typeColor(widget.item.type, widget.item.extension);
    final label = typeLabel(widget.item.type, widget.item.extension);

    return GestureDetector(
      onTap: _open,
      child: Container(
        height: 56,
        margin: const EdgeInsets.symmetric(horizontal: 12, vertical: 2),
        padding: const EdgeInsets.symmetric(horizontal: 12),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(12),
          color: widget.isSelected
              ? (isDark
                  ? Colors.white.withValues(alpha: 0.10)
                  : const Color(0xFFF0F0F0))
              : Colors.transparent,
          border: widget.isSelected
              ? Border.all(
                  color: AppColors.accent.withValues(alpha: 0.35),
                  width: 0.5,
                )
              : null,
        ),
        child: Row(
          children: [
            // Selection indicator bar
            if (widget.isSelected)
              Container(
                width: 3,
                height: 28,
                margin: const EdgeInsets.only(right: 9),
                decoration: BoxDecoration(
                  borderRadius: BorderRadius.circular(2),
                  color: AppColors.accent,
                ),
              ),
            // Type indicator dot
            Container(
              width: 6,
              height: 6,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: color,
                boxShadow: [
                  BoxShadow(
                    color: color.withValues(alpha: 0.5),
                    blurRadius: 4,
                    spreadRadius: -1,
                  ),
                ],
              ),
            ),
            const SizedBox(width: 10),
            // Icon / emoji
            SizedBox(
              width: 32,
              height: 32,
              child: _buildIcon(),
            ),
            const SizedBox(width: 10),
            // Name + path
            Expanded(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  _buildHighlightedName(isDark),
                  const SizedBox(height: 2),
                  Row(
                    children: [
                      _buildTypeBadge(label, isDark),
                      const SizedBox(width: 6),
                      Expanded(
                        child: Text(
                          widget.item.path,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: TextStyle(
                            fontSize: 11,
                            color: isDark
                                ? AppColors.textTertiaryDark
                                : AppColors.textTertiaryLight,
                          ),
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
            // Open hint on selection
            if (widget.isSelected)
              Container(
                width: 20,
                height: 20,
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
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildIcon() {
    if (_iconBytes != null && _iconBytes!.isNotEmpty) {
      return ClipRRect(
        borderRadius: BorderRadius.circular(6),
        child: Image.memory(
          _iconBytes!,
          width: 32,
          height: 32,
          fit: BoxFit.cover,
          errorBuilder: (_, _, _) => _emojiFallback(),
        ),
      );
    }
    return _emojiFallback();
  }

  Widget _emojiFallback() {
    final emoji = _emojiForType();
    if (emoji.isEmpty) {
      return Container(
        width: 32,
        height: 32,
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(6),
          color: typeColor(widget.item.type, widget.item.extension)
              .withValues(alpha: 0.12),
        ),
        child: const Icon(Icons.apps_rounded, size: 18, color: AppColors.typeApp),
      );
    }
    return Center(
      child: Text(emoji, style: const TextStyle(fontSize: 20)),
    );
  }

  Widget _buildHighlightedName(bool isDark) {
    final name = widget.item.name;
    final query = widget.query;

    if (query.isEmpty || !name.toLowerCase().contains(query.toLowerCase())) {
      return Text(
        name,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
        style: TextStyle(
          fontSize: 13.5,
          fontWeight: FontWeight.w600,
          color: isDark ? AppColors.textPrimaryDark : AppColors.textPrimaryLight,
        ),
      );
    }

    // Highlight the matched substring
    final lowerName = name.toLowerCase();
    final idx = lowerName.indexOf(query.toLowerCase());

    return RichText(
      maxLines: 1,
      overflow: TextOverflow.ellipsis,
      text: TextSpan(
        style: TextStyle(
          fontSize: 13.5,
          fontWeight: FontWeight.w600,
          color: isDark ? AppColors.textPrimaryDark : AppColors.textPrimaryLight,
        ),
        children: [
          if (idx > 0) TextSpan(text: name.substring(0, idx)),
          TextSpan(
            text: name.substring(idx, idx + query.length),
            style: TextStyle(
              backgroundColor: AppColors.accent.withValues(alpha: 0.2),
              color: isDark ? AppColors.accentLight : AppColors.accentDark,
            ),
          ),
          if (idx + query.length < name.length)
            TextSpan(
                text: name.substring(idx + query.length)),
        ],
      ),
    );
  }

  Widget _buildTypeBadge(String label, bool isDark) {
    final color = typeColor(widget.item.type, widget.item.extension);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 1),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(4),
        color: isDark
            ? color.withValues(alpha: 0.15)
            : color.withValues(alpha: 0.1),
      ),
      child: Text(
        label,
        style: TextStyle(
          fontSize: 10,
          fontWeight: FontWeight.w600,
          color: isDark ? color.withValues(alpha: 0.9) : color,
        ),
      ),
    );
  }
}
