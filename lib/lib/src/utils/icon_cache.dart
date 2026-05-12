import 'dart:async';
import 'dart:convert';
import 'dart:developer' as developer;
import 'dart:typed_data';
import '../rust/api/app.dart' as rust_app;
import '../rust/api/file.dart' as rust_file;

/// Shared cache for loaded icons, keyed by path.
class IconCache {
  static final Map<String, Uint8List> _cache = {};
  static final Map<String, Future<Uint8List?>> _pending = {};
  static final Set<String> _failed = {};
  static final Map<String, int> _retryCount = {};
  static const int _maxRetries = 2;

  static Uint8List? get(String path) => _cache[path];

  static bool hasFailed(String path) => _failed.contains(path);

  static Future<Uint8List?> loadAppIcon(String path) async {
    if (_cache.containsKey(path)) return _cache[path];
    if (_failed.contains(path)) return null;
    if (_pending.containsKey(path)) return _pending[path];
    final future = _doLoadAppIcon(path);
    _pending[path] = future;
    try {
      return await future;
    } finally {
      _pending.remove(path);
    }
  }

  static Future<Uint8List?> _doLoadAppIcon(String path) async {
    try {
      final icon = await rust_app.getAppIconBase64(path: path);
      if (icon != null && icon.isNotEmpty) {
        final bytes = _decode(icon);
        _cache[path] = bytes;
        _failed.remove(path);
        _retryCount.remove(path);
        return bytes;
      }
    } catch (e) {
      developer.log('Icon load failed: $path', name: 'IconCache', error: e);
    }
    final count = (_retryCount[path] ?? 0) + 1;
    _retryCount[path] = count;
    if (count >= _maxRetries) {
      _failed.add(path);
      _retryCount.remove(path);
    }
    return null;
  }

  static Future<Uint8List?> loadFileThumb(String path) async {
    final key = 'thumb:$path';
    if (_cache.containsKey(key)) return _cache[key];
    if (_failed.contains(key)) return null;
    if (_pending.containsKey(key)) return _pending[key];
    final future = _doLoadFileThumb(path, key);
    _pending[key] = future;
    try {
      return await future;
    } finally {
      _pending.remove(key);
    }
  }

  static Future<Uint8List?> _doLoadFileThumb(String path, String key) async {
    try {
      final thumb = await rust_file.getImageThumbnail(path: path);
      if (thumb != null && thumb.isNotEmpty) {
        final bytes = _decode(thumb);
        _cache[key] = bytes;
        _failed.remove(key);
        return bytes;
      }
    } catch (e) {
      developer.log('Thumb load failed: $path', name: 'IconCache', error: e);
    }
    _failed.add(key);
    return null;
  }

  static Future<void> preload(List<String> paths) async {
    await Future.wait(paths.map((p) => loadAppIcon(p)));
  }

  static Uint8List _decode(String dataUri) {
    final parts = dataUri.split(',');
    return base64.decode(parts.length == 2 ? parts[1] : dataUri);
  }
}
