import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../rust/api/clipboard.dart' as rust_cb;
import '../rust/clipboard_impl/history.dart';

class ClipboardState {
  final List<ClipboardItem> items;
  final int selectedIndex;
  final String query;

  const ClipboardState({
    this.items = const [],
    this.selectedIndex = 0,
    this.query = '',
  });

  ClipboardState copyWith({
    List<ClipboardItem>? items,
    int? selectedIndex,
    String? query,
    bool clearQuery = false,
  }) {
    return ClipboardState(
      items: items ?? this.items,
      selectedIndex: selectedIndex ?? this.selectedIndex,
      query: clearQuery ? '' : (query ?? this.query),
    );
  }

  List<ClipboardItem> get filteredItems {
    if (query.isEmpty) return items;
    final lower = query.toLowerCase();
    return items.where((i) => i.preview.toLowerCase().contains(lower)).toList();
  }
}

class ClipboardNotifier extends StateNotifier<ClipboardState> {
  ClipboardNotifier() : super(const ClipboardState());

  Future<void> loadHistory() async {
    try {
      final items = await rust_cb.getClipboardHistory();
      state = state.copyWith(items: items);
    } catch (_) {}
  }

  void search(String keyword) {
    if (keyword.isEmpty) {
      state = state.copyWith(clearQuery: true, selectedIndex: 0);
    } else {
      state = state.copyWith(query: keyword, selectedIndex: 0);
    }
  }

  Future<bool> copySelected() async {
    final items = state.filteredItems;
    if (items.isEmpty) return false;
    return copyClipboardItem(items[state.selectedIndex]);
  }

  /// Copy a specific clipboard item (used by search results).
  Future<bool> copyClipboardItem(ClipboardItem item) async {
    try {
      if (item.contentType == 'image') {
        final full = await rust_cb.getClipboardImageFullBase64(id: item.id);
        if (full != null) {
          final base64 = full.contains(',') ? full.split(',').last : full;
          await rust_cb.copyImageToClipboard(base64Data: base64);
        }
      } else if (item.content != null) {
        await rust_cb.copyToClipboard(content: item.content!);
      } else {
        await rust_cb.copyToClipboard(content: item.preview);
      }
      return true;
    } catch (_) {
      return false;
    }
  }

  Future<void> deleteItem(String id) async {
    await rust_cb.deleteClipboardItem(id: id);
    await loadHistory();
  }

  Future<void> togglePin(String id) async {
    await rust_cb.togglePinClipboardItem(id: id);
    await loadHistory();
  }

  Future<void> clearAll() async {
    await rust_cb.clearClipboardHistory();
    state = const ClipboardState();
  }

  void selectIndex(int index) {
    final items = state.filteredItems;
    if (items.isEmpty) return;
    state = state.copyWith(selectedIndex: index.clamp(0, items.length - 1));
  }

  void selectNext() {
    final items = state.filteredItems;
    if (items.isEmpty) return;
    state = state.copyWith(
      selectedIndex: (state.selectedIndex + 1) % items.length,
    );
  }

  void selectPrev() {
    final items = state.filteredItems;
    if (items.isEmpty) return;
    state = state.copyWith(
      selectedIndex:
          (state.selectedIndex - 1 + items.length) % items.length,
    );
  }
}

final clipboardProvider =
    StateNotifierProvider<ClipboardNotifier, ClipboardState>((ref) {
  return ClipboardNotifier();
});
