import 'dart:async';
import 'dart:io' show Platform;
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:window_manager/window_manager.dart';
import 'package:tray_manager/tray_manager.dart';
import 'src/rust/frb_generated.dart';
import 'src/rust/api/window.dart' as rust_window;
import 'src/rust/api/app.dart' as rust_app;
import 'app.dart';

const double windowWidth = 720;
const double windowHeight = 520;

late final String exeDir;

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Limit image cache memory (~20 MB decoded images)
  PaintingBinding.instance.imageCache.maximumSizeBytes = 20 << 20;
  PaintingBinding.instance.imageCache.maximumSize = 256;

  exeDir = Platform.resolvedExecutable.substring(
      0, Platform.resolvedExecutable.lastIndexOf(Platform.pathSeparator));

  await RustLib.init();

  // 初始化 Rust 后端（剪贴板监听 + 全局热键）
  await rust_window.initBackend();

  // ── 窗口设置 ──
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
  // Start hidden — set opacity 0 so window initializes invisibly
  await windowManager.setOpacity(0.0);

  // 拦截关闭事件：隐藏窗口而非退出
  await windowManager.setPreventClose(true);
  windowManager.addListener(_AppWindowListener());

  // ── 系统托盘 ──
  try {
    await _initSystemTray();
  } catch (_) {
    // 托盘初始化失败不阻塞启动
  }

  // ── 全局热键轮询：Alt+Space 切换窗口显隐 ──
  Timer.periodic(const Duration(milliseconds: 50), (_) async {
    final shouldToggle = await rust_window.consumeHotkeyFlag();
    if (shouldToggle) {
      await _toggleWindow();
    }
  });

  runApp(const ProviderScope(child: WToolsApp()));
  // After first frame, hide properly and restore opacity for future use
  await Future.delayed(const Duration(milliseconds: 300));
  await windowManager.hide();
  await windowManager.blur();
  await windowManager.setOpacity(1.0);
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
  }
}

/// 初始化系统托盘
Future<void> _initSystemTray() async {
  await trayManager.setIcon('assets/icon.ico');
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
}

class _TrayListener extends TrayListener {
  @override
  void onTrayIconMouseDown() {
    _toggleWindow();
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
