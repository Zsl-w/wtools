import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:window_manager/window_manager.dart';
import '../models/search_result.dart';
import '../providers/search_provider.dart';
import '../providers/clipboard_provider.dart';
import '../providers/settings_provider.dart';
import '../rust/api/app.dart' as rust_app;
import '../theme/app_theme.dart';
import '../widgets/glass_container.dart';
import '../widgets/search_bar_widget.dart';
import '../widgets/result_list_widget.dart';
import '../widgets/clipboard_list_widget.dart';

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

  @override
  void initState() {
    super.initState();
    ref.read(searchProvider);

    // Auto-focus search bar on startup
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

  KeyEventResult _handleKeyDown(KeyDownEvent event) {
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
    final state = ref.read(searchProvider);

    late final SearchResult item;
    if (state.focusArea == FocusArea.appRow) {
      if (state.appResults.isEmpty) return;
      item = state.appResults[state.selectedAppIndex];
    } else {
      if (state.fileResults.isEmpty) return;
      item = state.fileResults[state.selectedFileIndex];
    }

    try {
      if (item.type == 'app') {
        await rust_app.launchApp(path: item.path);
      } else {
        await rust_app.openFile(path: item.path);
      }
      await _hideWindow();
    } catch (_) {}
  }

  Future<void> _copyClipboardAndHide() async {
    final success = await ref.read(clipboardProvider.notifier).copySelected();
    if (success) await _hideWindow();
  }

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return Focus(
      autofocus: true,
      onFocusChange: (hasFocus) {
        if (!hasFocus) _hideWindow();
      },
      onKeyEvent: (node, event) {
        if (event is KeyDownEvent) return _handleKeyDown(event);
        return KeyEventResult.ignored;
      },
      child: Scaffold(
        backgroundColor:
            isDark ? const Color(0xFF1C1C1E) : const Color(0xFFF5F5F5),
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
                    child: SearchBarWidget(key: _searchBarKey),
                  ),
                  // Mode indicator
                  _buildModeIndicator(isDark),
                  // Divider
                  Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 24),
                    child: Divider(
                      height: 1,
                      color: Colors.black.withValues(alpha: isDark ? 0.08 : 0.04),
                    ),
                  ),
                  // ── Content area ──
                  Expanded(
                    child: _mode == _AppMode.search
                        ? const ResultListWidget()
                        : const ClipboardListWidget(),
                  ),
                  // ── Bottom bar ──
                  _buildBottomBar(isDark),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildModeIndicator(bool isDark) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(24, 6, 24, 6),
      child: Row(
        children: [
          _modeTab('🔍 搜索', _AppMode.search, isDark),
          const SizedBox(width: 4),
          _modeTab('📋 剪贴板', _AppMode.clipboard, isDark),
          const Spacer(),
          // Theme toggle
          GestureDetector(
            onTap: () => ref.read(settingsProvider.notifier).cycleTheme(),
            child: Container(
              width: 26,
              height: 26,
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(6),
                color: isDark
                    ? Colors.white.withValues(alpha: 0.04)
                    : Colors.black.withValues(alpha: 0.04),
              ),
              child: Icon(
                isDark ? Icons.light_mode_rounded : Icons.dark_mode_rounded,
                size: 14,
                color: isDark ? AppColors.accentLight : AppColors.accent,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _modeTab(String label, _AppMode mode, bool isDark) {
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
            color: active
                ? (isDark ? AppColors.accentLight : AppColors.accent)
                : (isDark ? AppColors.textTertiaryDark : AppColors.textTertiaryLight),
          ),
        ),
      ),
    );
  }

  Widget _buildBottomBar(bool isDark) {
    return Container(
      height: 34,
      padding: const EdgeInsets.symmetric(horizontal: 24),
      decoration: BoxDecoration(
        border: Border(
          top: BorderSide(
            color: Colors.black.withValues(alpha: isDark ? 0.06 : 0.03),
          ),
        ),
      ),
      child: Row(
        children: [
          _bottomHint('←→', '应用', isDark),
          const SizedBox(width: 12),
          _bottomHint('↑↓', '文件', isDark),
          const SizedBox(width: 12),
          _bottomHint('Enter', '打开', isDark),
          const SizedBox(width: 12),
          _bottomHint('Tab', '切换', isDark),
          const SizedBox(width: 12),
          _bottomHint('Esc', '关闭', isDark),
          const Spacer(),
          _bottomHint('Alt+Space', '唤起', isDark),
        ],
      ),
    );
  }

  Widget _bottomHint(String key, String action, bool isDark) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 2),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(4),
            color: isDark
                ? Colors.white.withValues(alpha: 0.06)
                : Colors.black.withValues(alpha: 0.05),
          ),
          child: Text(
            key,
            style: TextStyle(
              fontSize: 10,
              fontWeight: FontWeight.w600,
              color: isDark
                  ? AppColors.textSecondaryDark
                  : AppColors.textSecondaryLight,
            ),
          ),
        ),
        const SizedBox(width: 4),
        Text(
          action,
          style: TextStyle(
            fontSize: 10,
            color: isDark
                ? AppColors.textTertiaryDark
                : AppColors.textTertiaryLight,
          ),
        ),
      ],
    );
  }
}
