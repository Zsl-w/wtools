import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'src/providers/settings_provider.dart';
import 'src/theme/app_theme.dart';
import 'src/pages/main_window_page.dart';

class WToolsApp extends ConsumerWidget {
  const WToolsApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final settings = ref.watch(settingsProvider);

    return MaterialApp(
      title: 'wTools',
      debugShowCheckedModeBanner: false,
      theme: AppTheme.light(),
      darkTheme: AppTheme.dark(),
      themeMode: settings.flutterThemeMode,
      home: const MainWindowPage(),
    );
  }
}
