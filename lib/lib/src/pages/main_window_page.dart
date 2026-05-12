import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:window_manager/window_manager.dart';
import '../models/search_result.dart';
import '../providers/search_provider.dart';
import '../providers/clipboard_provider.dart';
import '../rust/api/app.dart' as rust_app;
import '../theme/app_theme.dart';
import '../widgets/glass_container.dart';
import '../widgets/search_bar_widget.dart';
import '../widgets/result_list_widget.dart';
import '../widgets/clipboard_list_widget.dart';
import '../utils/window_events.dart';

enum _AppMode { search, clipboard }

class MainWindowPage extends ConsumerStatefulWidget {
  const MainWindowPage({super.key});

  @override
  ConsumerState<MainWindowPage> createState() => _MainWindowPageState();
}

class _MainWindowPageState extends ConsumerState<MainWindowPage>
    with SingleTickerProviderStateMixin {
  final _searchBarKey = GlobalKey<SearchBarWidgetState>();
  late final AnimationController _entranceController;
  late final Animation<double> _scaleAnim;
  late final Animation<double> _fadeAnim;

  var _mode = _AppMode.search;
  var _isOpening = false;

  @override
  void initState() {
    super.initState();
    ref.read(searchProvider);

    // Auto-focus search bar on startup and when window is shown
    onWindowShown = () => _searchBarKey.currentState?.focusInput();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _searchBarKey.currentState?.focusInput();
    });

    _entranceController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 150),
    );
    _scaleAnim = CurvedAnimation(parent: _entranceController, curve: Curves.easeOutCubic);
    _fadeAnim = CurvedAnimation(
      parent: _entranceController,
      curve: const Interval(0.0, 0.6, curve: Curves.easeOut),
    );
    _entranceController.forward();
  }

  @override
  void dispose() {
    _entranceController.dispose();
    super.dispose();
  }

  Future<void> _hideWindow() async {
    await windowManager.hide();
    await windowManager.blur();
  }

  Future<void> _quitApp() async {
    await windowManager.hide();
    await windowManager.blur();
    await windowManager.destroy();
  }

  KeyEventResult _handleKeyDown(KeyDownEvent event) {
    // Alt+F4 quits the app
    if (event.logicalKey == LogicalKeyboardKey.f4 &&
        HardwareKeyboard.instance.isAltPressed) {
      _quitApp();
      return KeyEventResult.handled;
    }
    // Tab switches mode
    if (event.logicalKey == LogicalKeyboardKey.tab) {
      setState(() {
        _mode = _mode == _AppMode.search
            ? _AppMode.clipboard
            : _AppMode.search;
      });
      return KeyEventResult.handled;
    }

    if (event.logicalKey == LogicalKeyboardKey.escape) {
      _hideWindow();
      return KeyEventResult.handled;
    }

    if (_mode == _AppMode.search) {
      return _handleSearchKey(event);
    } else {
      return _handleClipboardKey(event);
    }
  }

  KeyEventResult _handleSearchKey(KeyDownEvent event) {
    final notifier = ref.read(searchProvider.notifier);
    final state = ref.read(searchProvider);

    switch (event.logicalKey) {
      case LogicalKeyboardKey.arrowLeft:
        notifier.selectPrevApp();
        return KeyEventResult.handled;
      case LogicalKeyboardKey.arrowRight:
        notifier.selectNextApp();
        return KeyEventResult.handled;
      case LogicalKeyboardKey.arrowDown:
        if (state.focusArea == FocusArea.appRow) {
          if (state.fileResults.isNotEmpty) {
            notifier.moveFocusToFileList();
          }
        } else {
          notifier.selectNextFile();
        }
        return KeyEventResult.handled;
      case LogicalKeyboardKey.arrowUp:
        if (state.focusArea == FocusArea.fileList) {
          if (state.selectedFileIndex == 0) {
            if (state.appResults.isNotEmpty) {
              notifier.moveFocusToAppRow();
            }
          } else {
            notifier.selectPrevFile();
          }
        }
        return KeyEventResult.handled;
      case LogicalKeyboardKey.enter:
      case LogicalKeyboardKey.numpadEnter:
        _openSelectedSearch();
        return KeyEventResult.handled;
      default:
        return KeyEventResult.ignored;
    }
  }

  KeyEventResult _handleClipboardKey(KeyDownEvent event) {
    final notifier = ref.read(clipboardProvider.notifier);
    switch (event.logicalKey) {
      case LogicalKeyboardKey.arrowDown:
        notifier.selectNext();
        return KeyEventResult.handled;
      case LogicalKeyboardKey.arrowUp:
        notifier.selectPrev();
        return KeyEventResult.handled;
      case LogicalKeyboardKey.enter:
      case LogicalKeyboardKey.numpadEnter:
        _copyClipboardAndHide();
        return KeyEventResult.handled;
      default:
        return KeyEventResult.ignored;
    }
  }

  Future<void> _openSelectedSearch() async {
    if (_isOpening) return;
    _isOpening = true;

    final state = ref.read(searchProvider);

    late final SearchResult item;
    if (state.focusArea == FocusArea.appRow) {
      if (state.appResults.isEmpty) {
        _isOpening = false;
        return;
      }
      item = state.appResults[state.selectedAppIndex];
    } else {
      if (state.fileResults.isEmpty) {
        _isOpening = false;
        return;
      }
      item = state.fileResults[state.selectedFileIndex];
    }

    try {
      if (item.type == 'app') {
        await rust_app.launchApp(path: item.path);
      } else {
        await rust_app.openFile(path: item.path);
      }
      await _hideWindow();
    } catch (_) {
    } finally {
      _isOpening = false;
    }
  }

  Future<void> _copyClipboardAndHide() async {
    final success = await ref.read(clipboardProvider.notifier).copySelected();
    if (success) await _hideWindow();
  }

  @override
  Widget build(BuildContext context) {
    return Focus(
      autofocus: true,
      onKeyEvent: (node, event) {
        if (event is KeyDownEvent) return _handleKeyDown(event);
        return KeyEventResult.ignored;
      },
      child: Scaffold(
        backgroundColor: const Color(0xFFF5F3EE),
        body: ScaleTransition(
          scale: Tween<double>(begin: 0.94, end: 1.0).animate(_scaleAnim),
          child: FadeTransition(
            opacity: _fadeAnim,
            child: GlassContainer(
              width: double.infinity,
              height: double.infinity,
              borderRadius: BorderRadius.zero,
              child: Column(
                children: [
                  // ── Search bar + mode indicator ──
                  Padding(
                    padding: const EdgeInsets.fromLTRB(16, 12, 16, 0),
                    child: SearchBarWidget(
                      key: _searchBarKey,
                      onSubmitted: _openSelectedSearch,
                    ),
                  ),
                  // Mode indicator
                  _buildModeIndicator(),
                  // Divider
                  Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 24),
                    child: Divider(
                      height: 1,
                      color: Colors.black.withValues(alpha: 0.04),
                    ),
                  ),
                  // ── Content area ──
                  Expanded(
                    child: _mode == _AppMode.search
                        ? const ResultListWidget()
                        : const ClipboardListWidget(),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildModeIndicator() {
    return Padding(
      padding: const EdgeInsets.fromLTRB(24, 6, 24, 6),
      child: Row(
        children: [
          _modeTab('🔍 搜索', _AppMode.search),
          const SizedBox(width: 4),
          _modeTab('📋 剪贴板', _AppMode.clipboard),
        ],
      ),
    );
  }

  Widget _modeTab(String label, _AppMode mode) {
    final active = _mode == mode;
    return GestureDetector(
      onTap: () => setState(() => _mode = mode),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 120),
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(8),
          color: active
              ? AppColors.accent.withValues(alpha: 0.12)
              : Colors.transparent,
        ),
        child: Text(
          label,
          style: TextStyle(
            fontSize: 12,
            fontWeight: active ? FontWeight.w600 : FontWeight.w400,
            color: active ? AppColors.accent : AppColors.textTertiary,
          ),
        ),
      ),
    );
  }
}
