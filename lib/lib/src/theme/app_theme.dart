import 'package:flutter/material.dart';

/// wTools 配色系统：克莱因蓝 + 日落橙
///
/// 克莱因蓝 (#002FAF) — 深邃、理性的蓝色，用于主要强调色
/// 日落橙 (#F26419) — 温暖、活力的橙色，用于次要强调色/类型标识
class AppColors {
  AppColors._();

  // ── 克莱因蓝 (Klein Blue) ──
  static const primary = Color(0xFF002FAF);
  static const primaryLight = Color(0xFF3356C3);
  static const primaryDark = Color(0xFF002080);
  static const primaryMuted = Color(0xFFB0C4FF);

  // ── 日落橙 (Sunset Orange) ──
  static const secondary = Color(0xFFF26419);
  static const secondaryLight = Color(0xFFFF8C42);
  static const secondaryDark = Color(0xFFCC5200);
  static const secondaryMuted = Color(0xFFFFD6B0);

  // ── 别名（兼容旧代码，指向克莱因蓝） ──
  static const accent = primary;
  static const accentDark = primaryDark;
  static const accentMuted = primaryMuted;

  // ── 玻璃质感表面 ──
  static Color glassLight = const Color(0xB8FFFFFF);
  static Color glassBorderLight = const Color(0x80FFFFFF);

  // ── 类型标识颜色 ──
  static const typeApp = Color(0xFF002FAF);    // 克莱因蓝
  static const typeFile = Color(0xFF10B981);   // 绿色（保持辨识度）
  static const typeFolder = Color(0xFFF26419); // 日落橙
  static const typeDoc = Color(0xFF3356C3);    // 克莱因蓝浅色
  static const typeImage = Color(0xFFEC4899);  // 粉红（保持辨识度）
  static const typeMedia = Color(0xFFF97316);  // 橙色
  static const typeLink = Color(0xFF3B82F6);   // 蓝色

  // ── 文本 ──
  static const textPrimary = Color(0xFF1A1A1A);
  static const textSecondary = Color(0xFF737373);
  static const textTertiary = Color(0xFFA3A3A3);
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
}

/// 文件类型注册表：扩展名 → (颜色, 中文标签)
const _fileTypeRegistry = <String, (Color color, String label)>{
  // 文档
  'pdf':  (AppColors.typeDoc, 'PDF'),
  'doc':  (AppColors.typeDoc, '文档'),
  'docx': (AppColors.typeDoc, '文档'),
  'xls':  (AppColors.typeDoc, '表格'),
  'xlsx': (AppColors.typeDoc, '表格'),
  'ppt':  (AppColors.typeDoc, '演示'),
  'pptx': (AppColors.typeDoc, '演示'),
  'txt':  (AppColors.typeDoc, '文本'),
  'md':   (AppColors.typeDoc, '文本'),
  // 图片
  'png':  (AppColors.typeImage, '图片'),
  'jpg':  (AppColors.typeImage, '图片'),
  'jpeg': (AppColors.typeImage, '图片'),
  'gif':  (AppColors.typeImage, '图片'),
  'webp': (AppColors.typeImage, '图片'),
  'svg':  (AppColors.typeImage, '图片'),
  // 媒体
  'mp3':  (AppColors.typeMedia, '音频'),
  'wav':  (AppColors.typeMedia, '音频'),
  'flac': (AppColors.typeMedia, '音频'),
  'mp4':  (AppColors.typeMedia, '视频'),
  'avi':  (AppColors.typeMedia, '视频'),
  'mkv':  (AppColors.typeMedia, '视频'),
  // 代码
  'js':   (AppColors.typeLink, '代码'),
  'ts':   (AppColors.typeLink, '代码'),
  'py':   (AppColors.typeLink, '代码'),
  'rs':   (AppColors.typeLink, '代码'),
  'go':   (AppColors.typeLink, '代码'),
  'html': (AppColors.typeLink, '代码'),
  'css':  (AppColors.typeLink, '代码'),
  // 压缩包
  'zip':  (AppColors.typeFile, '压缩'),
  'rar':  (AppColors.typeFile, '压缩'),
  '7z':   (AppColors.typeFile, '压缩'),
};

/// Type indicator color by file type
Color typeColor(String? resultType, String? extension) {
  if (resultType == 'app') return AppColors.typeApp;
  if (resultType == 'folder') return AppColors.typeFolder;
  if (extension == null) return AppColors.typeFile;
  return _fileTypeRegistry[extension.toLowerCase()]?.$1 ?? AppColors.typeFile;
}

/// Chinese type label for file extensions
String typeLabel(String? resultType, String? extension) {
  if (resultType == 'app') return '应用';
  if (resultType == 'folder') return '文件夹';
  if (extension == null) return '文件';
  return _fileTypeRegistry[extension.toLowerCase()]?.$2 ?? extension.toUpperCase();
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
