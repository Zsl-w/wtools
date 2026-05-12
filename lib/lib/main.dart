import 'dart:async';
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart' show rootBundle;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:window_manager/window_manager.dart';
import 'package:tray_manager/tray_manager.dart';
import 'src/rust/frb_generated.dart';
import 'src/rust/api/window.dart' as rust_window;
import 'src/rust/api/app.dart' as rust_app;
import 'app.dart';
import 'src/utils/window_events.dart';

const double windowWidth = 720;
const double windowHeight = 520;

final _startupReady = Completer<void>();

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Limit image cache memory (~20 MB decoded images)
  PaintingBinding.instance.imageCache.maximumSizeBytes = 20 << 20;
  PaintingBinding.instance.imageCache.maximumSize = 256;

  await RustLib.init();

  // 初始化 Rust 后端（剪贴板监听 + 全局热键）
  await rust_window.initBackend();

  // ── 窗口设置 & 隐藏（runApp 前设置，引擎启动后生效） ──
  await windowManager.ensureInitialized();
  await windowManager.setAsFrameless();

  final options = WindowOptions(
    size: const Size(windowWidth, windowHeight),
    center: true,
    skipTaskbar: true,
    alwaysOnTop: true,
    backgroundColor: const Color(0xFF1C1C1E),
    titleBarStyle: TitleBarStyle.hidden,
    windowButtonVisibility: false,
  );

  await windowManager.waitUntilReadyToShow(options, () async {});

  // 拦截关闭事件：隐藏窗口而非退出
  await windowManager.setPreventClose(true);
  windowManager.addListener(_AppWindowListener());

  // ── 全局热键轮询：Alt+Space 切换窗口显隐 ──
  Timer.periodic(const Duration(milliseconds: 50), (_) async {
    final shouldToggle = await rust_window.consumeHotkeyFlag();
    if (shouldToggle) {
      await _toggleWindow();
    }
  });

  runApp(const ProviderScope(child: WToolsApp()));

  // 第一帧立即隐藏窗口（不依赖opacity，避免黑闪）
  WidgetsBinding.instance.addPostFrameCallback((_) {
    _startupReady.complete();
  });
  await _startupReady.future;
  await windowManager.hide();
  await windowManager.blur();

  // ── 系统托盘（在 runApp 之后初始化，确保 asset bundle 就绪） ──
  try {
    await _initSystemTray();
  } catch (e) {
    // 托盘初始化失败不阻塞启动
  }
}

/// 切换窗口显隐
Future<void> _toggleWindow() async {
  final isVisible = await windowManager.isVisible();
  if (isVisible) {
    await windowManager.hide();
    await windowManager.blur();
  } else {
    await windowManager.show();
    await windowManager.focus();
    onWindowShown?.call();
  }
}

/// 初始化系统托盘
Future<void> _initSystemTray() async {
  // Destroy any stale tray icon from a previous run (prevents ghost icons)
  try { await trayManager.destroy(); } catch (_) {}
  await Future.delayed(const Duration(milliseconds: 100));

  // Save icon from asset bundle to temp file (tray_manager requires a path)
  final iconData = await rootBundle.load('assets/icon.ico');
  final tempIcon = File('${Directory.systemTemp.path}\\wtools_icon.ico');
  await tempIcon.writeAsBytes(iconData.buffer.asUint8List());
  await trayManager.setIcon(tempIcon.path);
  await trayManager.setToolTip('wTools - 快速搜索');

  final autostartEnabled = await rust_app.isAutostartEnabled();
  final autostartLabel = autostartEnabled ? '✓ 开机自启动' : '开机自启动';

  final menu = Menu(
    items: [
      MenuItem(key: 'show', label: '显示搜索'),
      MenuItem.separator(),
      MenuItem(key: 'autostart', label: autostartLabel),
      MenuItem(key: 'quit', label: '退出'),
    ],
  );

  await trayManager.setContextMenu(menu);
  trayManager.addListener(_TrayListener());
}

class _AppWindowListener extends WindowListener {
  @override
  void onWindowClose() {
    windowManager.hide();
    windowManager.blur();
  }

  @override
  void onWindowBlur() {
    if (!isPickerOpen) {
      windowManager.hide();
      windowManager.blur();
    }
  }
}

class _TrayListener extends TrayListener {
  @override
  void onTrayIconMouseDown() {
    _toggleWindow();
  }

  @override
  void onTrayIconRightMouseDown() {
    _showTrayMenu();
  }

  @override
  void onTrayIconRightMouseUp() {
    _showTrayMenu();
  }

  Future<void> _showTrayMenu() async {
    try {
      await trayManager.popUpContextMenu();
    } catch (_) {}
  }

  @override
  void onTrayMenuItemClick(MenuItem menuItem) async {
    switch (menuItem.key) {
      case 'show':
        await _toggleWindow();
        break;
      case 'autostart':
        final current = await rust_app.isAutostartEnabled();
        await rust_app.setAutostart(enabled: !current);
        final newLabel = !current ? '✓ 开机自启动' : '开机自启动';
        await trayManager.setContextMenu(Menu(
          items: [
            MenuItem(key: 'show', label: '显示搜索'),
            MenuItem.separator(),
            MenuItem(key: 'autostart', label: newLabel),
            MenuItem(key: 'quit', label: '退出'),
          ],
        ));
        break;
      case 'quit':
        await rust_window.shutdownBackend();
        await trayManager.destroy();
        await windowManager.destroy();
        break;
    }
  }
}
