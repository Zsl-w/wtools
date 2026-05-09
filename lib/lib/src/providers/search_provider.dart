import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../rust/api/app.dart' as rust_app;
import '../rust/api/file.dart' as rust_file;
import '../models/search_result.dart';
import '../utils/fuzzy_search.dart';
import '../utils/debouncer.dart';
import '../utils/icon_cache.dart';

/// Which keyboard-focusable area is currently active.
enum FocusArea { appRow, fileList }

/// Search state — separate app results (horizontal row) from file results (vertical list).
class SearchState {
  final String query;
  final List<SearchResult> appResults;
  final List<SearchResult> fileResults;
  final int selectedAppIndex;
  final int selectedFileIndex;
  final FocusArea focusArea;
  final bool isLoading;
  final String? error;

  const SearchState({
    this.query = '',
    this.appResults = const [],
    this.fileResults = const [],
    this.selectedAppIndex = 0,
    this.selectedFileIndex = 0,
    this.focusArea = FocusArea.appRow,
    this.isLoading = false,
    this.error,
  });

  SearchState copyWith({
    String? query,
    List<SearchResult>? appResults,
    List<SearchResult>? fileResults,
    int? selectedAppIndex,
    int? selectedFileIndex,
    FocusArea? focusArea,
    bool? isLoading,
    String? error,
    bool clearError = false,
  }) {
    return SearchState(
      query: query ?? this.query,
      appResults: appResults ?? this.appResults,
      fileResults: fileResults ?? this.fileResults,
      selectedAppIndex: selectedAppIndex ?? this.selectedAppIndex,
      selectedFileIndex: selectedFileIndex ?? this.selectedFileIndex,
      focusArea: focusArea ?? this.focusArea,
      isLoading: isLoading ?? this.isLoading,
      error: clearError ? null : (error ?? this.error),
    );
  }
}

/// The search provider — orchestrates app indexing, two-phase search,
/// dual-zone keyboard navigation, and usage tracking.
class SearchNotifier extends StateNotifier<SearchState> {
  final Debouncer _debouncer = Debouncer(const Duration(milliseconds: 100));

  List<SearchableApp> _appIndex = [];
  Fuse<SearchableApp>? _fuse;
  Map<String, rust_app.AppUsage> _usage = {};
  bool _initialized = false;
  List<SearchResult>? _cachedTopApps;
  int _searchGeneration = 0;

  SearchNotifier() : super(const SearchState()) {
    _init();
  }

  Future<void> _init() async {
    final ok = await _buildIndex();
    if (ok) {
      _initialized = true;
      final topApps = _getTopApps(8);
      // Preload icons in background — cards will show fallback initially then update
      IconCache.preload(topApps.map((a) => a.path).toList()); // fire-and-forget
      state = state.copyWith(
        appResults: topApps,
        fileResults: const [],
        selectedAppIndex: 0,
        focusArea: FocusArea.appRow,
        error: null,
        clearError: true,
      );
    }
  }

  /// Reload apps and rebuild the search index. Called after adding/removing custom apps.
  Future<void> refreshIndex() async {
    await _buildIndex();
  }

  Future<bool> _buildIndex() async {
    try {
      final apps = await rust_app.getInstalledApps();
      final customApps = await rust_app.getCustomApps();
      _usage = await rust_app.getAppUsage();

      final allApps = <rust_app.AppInfo>[];
      for (final a in apps) {
        allApps.add(a);
      }
      for (final c in customApps) {
        if (!allApps.any((a) => a.path == c.path)) {
          allApps.add(rust_app.AppInfo(
            name: c.name,
            path: c.path,
            icon: null,
            aliases: [],
          ));
        }
      }

      _appIndex = allApps.map((a) {
        return SearchableApp(
          name: a.name,
          path: a.path,
          aliases: List<String>.from(a.aliases),
        );
      }).toList();

      _fuse = Fuse<SearchableApp>(
        _appIndex,
        options: const FuseOptions(
          keys: ['name', 'path', 'aliases'],
          threshold: 0.4,
        ),
      );
      _cachedTopApps = null;
      return true;
    } catch (e) {
      state = state.copyWith(error: e.toString(), clearError: false);
      return false;
    }
  }

  List<SearchResult> _getTopApps(int count) {
    if (_cachedTopApps != null) return _cachedTopApps!.take(count).toList();
    _cachedTopApps = _computeTopApps();
    return _cachedTopApps!.take(count).toList();
  }

  List<SearchResult> _computeTopApps() {
    final sorted = _appIndex.map((a) {
      final usage = _usage[a.path];
      return (a: a, count: usage?.count ?? 0, lastUsed: usage?.lastUsed ?? 0);
    }).toList()
      ..sort((a, b) {
        if (a.count != b.count) return b.count.compareTo(a.count);
        return b.lastUsed.compareTo(a.lastUsed);
      });
    return sorted.map((t) {
      return SearchResult.fromApp(rust_app.AppInfo(
        name: t.a.name,
        path: t.a.path,
        icon: null,
        aliases: t.a.aliases,
      ));
    }).toList();
  }

  /// Called on every keystroke in the search bar.
  void search(String keyword) {
    if (!_initialized) return;

    if (keyword.isEmpty) {
      _debouncer.cancel();
      final topApps = _getTopApps(8);
      state = SearchState(
        appResults: topApps,
        fileResults: const [],
        focusArea: FocusArea.appRow,
      );
      return;
    }

    // Only update query immediately — keep current results visible to avoid flash
    state = state.copyWith(query: keyword, clearError: true);
    _debouncer.run(() => _executeSearch(keyword));
  }

  Future<void> _executeSearch(String keyword) async {
    final generation = ++_searchGeneration;
    // Phase 1: sync app search via Fuse
    final fuseResults = _fuse?.search(keyword, limit: 8) ?? [];
    final appResults = fuseResults.map((r) {
      return SearchResult.fromApp(rust_app.AppInfo(
        name: r.item.name,
        path: r.item.path,
        icon: null,
        aliases: r.item.aliases,
      ));
    }).toList();

    if (generation != _searchGeneration) return;
    state = state.copyWith(
      appResults: appResults,
      selectedAppIndex: 0,
      focusArea: appResults.isNotEmpty ? FocusArea.appRow : FocusArea.fileList,
      isLoading: true,
      clearError: true,
    );

    // Phase 2: async file search via Everything
    try {
      final files = await rust_file.searchFiles(query: keyword, limit: 6);
      if (generation != _searchGeneration) return;
      final fileResults = files.map((f) => SearchResult.fromFile(f)).toList();

      final newFocus = (appResults.isEmpty && fileResults.isNotEmpty)
          ? FocusArea.fileList
          : state.focusArea;

      state = state.copyWith(
        fileResults: fileResults,
        isLoading: false,
        focusArea: newFocus,
        clearError: true,
      );
    } catch (e) {
      if (generation != _searchGeneration) return;
      state = state.copyWith(
        isLoading: false,
        error: '文件搜索失败: $e',
        clearError: false,
      );
    }
  }

  // ── Zone-navigation methods ──

  void selectNextApp() {
    if (state.appResults.isEmpty) return;
    state = state.copyWith(
      selectedAppIndex:
          (state.selectedAppIndex + 1) % state.appResults.length,
    );
  }

  void selectPrevApp() {
    if (state.appResults.isEmpty) return;
    state = state.copyWith(
      selectedAppIndex: (state.selectedAppIndex - 1 + state.appResults.length) %
          state.appResults.length,
    );
  }

  void selectNextFile() {
    if (state.fileResults.isEmpty) return;
    state = state.copyWith(
      selectedFileIndex:
          (state.selectedFileIndex + 1) % state.fileResults.length,
    );
  }

  void selectPrevFile() {
    if (state.fileResults.isEmpty) return;
    state = state.copyWith(
      selectedFileIndex:
          (state.selectedFileIndex - 1 + state.fileResults.length) %
              state.fileResults.length,
    );
  }

  void moveFocusToFileList() {
    if (state.fileResults.isEmpty) return;
    state = state.copyWith(
      focusArea: FocusArea.fileList,
      selectedFileIndex: 0,
    );
  }

  void moveFocusToAppRow() {
    if (state.appResults.isEmpty) return;
    state = state.copyWith(focusArea: FocusArea.appRow);
  }

  void clearSearch() {
    _debouncer.cancel();
    final topApps = _getTopApps(8);
    state = SearchState(appResults: topApps);
  }

  @override
  void dispose() {
    _debouncer.cancel();
    super.dispose();
  }
}

final searchProvider =
    StateNotifierProvider<SearchNotifier, SearchState>((ref) {
  return SearchNotifier();
});
