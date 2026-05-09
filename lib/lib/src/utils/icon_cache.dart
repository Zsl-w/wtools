import 'dart:convert';
import 'dart:typed_data';
import '../rust/api/app.dart' as rust_app;
import '../rust/api/file.dart' as rust_file;

/// Shared cache for loaded icons, keyed by path.
class IconCache {
  static final Map<String, Uint8List?> _cache = {};

  static Uint8List? get(String path) => _cache[path];

  static Future<Uint8List?> loadAppIcon(String path) async {
    if (_cache.containsKey(path)) return _cache[path];
    try {
      final icon = await rust_app.getAppIconBase64(path: path);
      if (icon != null && icon.isNotEmpty) {
        final bytes = _decode(icon);
        _cache[path] = bytes;
        return bytes;
      }
    } catch (_) {}
    _cache[path] = null;
    return null;
  }

  static Future<Uint8List?> loadFileThumb(String path) async {
    final key = 'thumb:$path';
    if (_cache.containsKey(key)) return _cache[key];
    try {
      final thumb = await rust_file.getImageThumbnail(path: path);
      if (thumb != null && thumb.isNotEmpty) {
        final bytes = _decode(thumb);
        _cache[key] = bytes;
        return bytes;
      }
    } catch (_) {}
    _cache[key] = null;
    return null;
  }

  /// Preload icons for given paths (fire-and-forget).
  static void preload(List<String> paths) {
    for (final path in paths) {
      if (!_cache.containsKey(path)) {
        loadAppIcon(path); // fire and forget
      }
    }
  }

  static Uint8List _decode(String dataUri) {
    final parts = dataUri.split(',');
    return base64.decode(parts.length == 2 ? parts[1] : dataUri);
  }
}
