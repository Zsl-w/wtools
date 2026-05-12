import 'package:flutter/material.dart';
import 'src/theme/app_theme.dart';
import 'src/pages/main_window_page.dart';

class WToolsApp extends StatelessWidget {
  const WToolsApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'wTools',
      debugShowCheckedModeBanner: false,
      theme: AppTheme.light(),
      home: const MainWindowPage(),
    );
  }
}
