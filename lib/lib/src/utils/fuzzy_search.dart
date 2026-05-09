/// A lightweight fuzzy search engine, roughly compatible with Fuse.js
/// behavior (threshold ~0.4).  Uses character-subsequence scoring.
class Fuse<T> {
  final List<T> _items;
  final FuseOptions _options;

  Fuse(this._items, {required FuseOptions options}) : _options = options;

  /// Returns up to [limit] matches, sorted by score ascending (lower = better).
  List<FuseResult<T>> search(String pattern, {int limit = 6}) {
    if (pattern.isEmpty) return [];

    final lower = pattern.toLowerCase();
    final results = <_ScoredItem<T>>[];

    for (int i = 0; i < _items.length; i++) {
      final item = _items[i];
      double best = double.infinity;

      for (final key in _options.keys) {
        final value = _getValue(item, key).toLowerCase();
        final score = _score(value, lower);
        if (score < best) best = score;
      }

      if (best < _options.threshold) {
        results.add(_ScoredItem<T>(item, best));
      }
    }

    results.sort((a, b) => a.score.compareTo(b.score));
    return results
        .take(limit)
        .map((r) => FuseResult<T>(r.item, r.score))
        .toList();
  }

  double _score(String text, String pattern) {
    int ti = 0, pi = 0;
    int firstMatch = -1;
    int lastMatch = -1;
    int gaps = 0;

    while (ti < text.length && pi < pattern.length) {
      if (text[ti] == pattern[pi]) {
        if (firstMatch == -1) firstMatch = ti;
        lastMatch = ti;
        pi++;
        ti++;
      } else if (pi > 0) {
        gaps++;
        ti++;
      } else {
        ti++;
      }
    }

    if (pi < pattern.length) return 1.0;

    final matchLen = lastMatch - firstMatch + 1;
    final gapPenalty = gaps / pattern.length;
    final startBonus = firstMatch == 0 ? 0.0 : (firstMatch / text.length) * 0.5;

    return ((matchLen - pattern.length) / pattern.length * 0.3 +
            gapPenalty * 0.4 +
            startBonus * 0.3)
        .clamp(0.0, 1.0);
  }

  String _getValue(T item, String key) {
    if (item is Searchable) return item.searchValue(key);
    return item.toString();
  }
}

class _ScoredItem<T> {
  final T item;
  final double score;
  const _ScoredItem(this.item, this.score);
}

/// Implement on data types to support multi-key search via Fuse.
abstract mixin class Searchable {
  String searchValue(String key);
}

class FuseOptions {
  final List<String> keys;
  final double threshold;

  const FuseOptions({required this.keys, this.threshold = 0.4});
}

class FuseResult<T> {
  final T item;
  final double score;

  const FuseResult(this.item, this.score);
}

// ── Concrete searchable wrapper ──

class SearchableApp with Searchable {
  final String name;
  final String path;
  final List<String> aliases;

  SearchableApp({
    required this.name,
    required this.path,
    this.aliases = const [],
  });

  @override
  String searchValue(String key) {
    switch (key) {
      case 'name':
        return name;
      case 'path':
        return path;
      case 'aliases':
        return aliases.join(' ');
      default:
        return name;
    }
  }
}
