import 'package:flutter/material.dart';

/// wTools Refined Glassmorphism Design System
///
/// A frosted-glass aesthetic inspired by physical materials —
/// layered translucency, edge highlights, and precise typography.
class AppColors {
  AppColors._();

  // ── Teal Accent ──
  static const accent = Color(0xFF14B8A6);
  static const accentLight = Color(0xFF5EEAD4);
  static const accentDark = Color(0xFF0F766E);
  static const accentMuted = Color(0xFF99F6E4);

  // ── Glass Surfaces ──
  static Color glassLight = const Color(0xB8FFFFFF); // ~72% white
  static Color glassDark = const Color(0xAA1C1C1E);
  static Color glassBorderLight = const Color(0x80FFFFFF); // 50% white
  static Color glassBorderDark = const Color(0x1AFFFFFF);

  // ── Type Indicators ──
  static const typeApp = Color(0xFFF59E0B);
  static const typeFile = Color(0xFF10B981);
  static const typeFolder = Color(0xFFF97316);
  static const typeDoc = Color(0xFF8B5CF6);
  static const typeImage = Color(0xFFEC4899);
  static const typeMedia = Color(0xFFF97316);
  static const typeLink = Color(0xFF3B82F6);

  // ── Text ──
  static const textPrimaryLight = Color(0xFF1A1A1A);
  static const textSecondaryLight = Color(0xFF737373);
  static const textTertiaryLight = Color(0xFFA3A3A3);
  static const textPrimaryDark = Color(0xFFFAFAFA);
  static const textSecondaryDark = Color(0xFF98989F);
  static const textTertiaryDark = Color(0xFF636366);
}

class AppTheme {
  AppTheme._();

  static const _fontFamily = 'HarmonyOS Sans SC';

  static ThemeData light() {
    return ThemeData(
      brightness: Brightness.light,
      fontFamily: _fontFamily,
      scaffoldBackgroundColor: Colors.transparent,
      textSelectionTheme: TextSelectionThemeData(
        cursorColor: AppColors.accent,
        selectionColor: AppColors.accent.withValues(alpha: 0.3),
        selectionHandleColor: AppColors.accent,
      ),
    );
  }

  static ThemeData dark() {
    return ThemeData(
      brightness: Brightness.dark,
      fontFamily: _fontFamily,
      scaffoldBackgroundColor: Colors.transparent,
      textSelectionTheme: TextSelectionThemeData(
        cursorColor: AppColors.accentLight,
        selectionColor: AppColors.accentLight.withValues(alpha: 0.3),
        selectionHandleColor: AppColors.accentLight,
      ),
    );
  }
}

/// Type indicator color by file type
Color typeColor(String? resultType, String? extension) {
  if (resultType == 'app') return AppColors.typeApp;
  if (resultType == 'folder') return AppColors.typeFolder;
  if (extension == null) return AppColors.typeFile;

  const docExts = ['pdf', 'doc', 'docx', 'txt', 'md', 'xls', 'xlsx', 'ppt', 'pptx'];
  const imageExts = ['png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'svg', 'ico'];
  const mediaExts = ['mp3', 'wav', 'flac', 'mp4', 'avi', 'mkv', 'mov'];
  const codeExts = [
    'js', 'ts', 'jsx', 'tsx', 'py', 'rs', 'go', 'rb', 'java',
    'c', 'cpp', 'h', 'cs', 'swift', 'kt', 'vue', 'svelte', 'dart',
    'html', 'css', 'scss', 'json', 'xml', 'yaml', 'toml', 'sql',
  ];

  if (imageExts.contains(extension)) return AppColors.typeImage;
  if (mediaExts.contains(extension)) return AppColors.typeMedia;
  if (docExts.contains(extension)) return AppColors.typeDoc;
  if (codeExts.contains(extension)) return AppColors.typeLink;
  return AppColors.typeFile;
}

/// Chinese type label for file extensions
String typeLabel(String? resultType, String? extension) {
  if (resultType == 'app') return '应用';
  if (resultType == 'folder') return '文件夹';
  if (extension == null) return '文件';

  const labels = {
    'pdf': 'PDF', 'doc': '文档', 'docx': '文档',
    'xls': '表格', 'xlsx': '表格', 'ppt': '演示', 'pptx': '演示',
    'png': '图片', 'jpg': '图片', 'jpeg': '图片', 'gif': '图片',
    'webp': '图片', 'svg': '图片',
    'mp3': '音频', 'wav': '音频', 'flac': '音频',
    'mp4': '视频', 'avi': '视频', 'mkv': '视频',
    'zip': '压缩', 'rar': '压缩', '7z': '压缩',
    'txt': '文本', 'md': '文本', 'json': '代码',
    'js': '代码', 'ts': '代码', 'py': '代码', 'rs': '代码',
    'go': '代码', 'html': '代码', 'css': '代码',
  };
  return labels[extension] ?? extension.toUpperCase();
}

/// Format file size
String formatSize(int bytes) {
  if (bytes < 1024) return '$bytes B';
  if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(1)} KB';
  if (bytes < 1024 * 1024 * 1024) {
    return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
  }
  return '${(bytes / (1024 * 1024 * 1024)).toStringAsFixed(1)} GB';
}
