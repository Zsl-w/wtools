import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

enum AppThemeMode { light, dark, system }

class SettingsState {
  final AppThemeMode themeMode;

  const SettingsState({this.themeMode = AppThemeMode.system});

  SettingsState copyWith({AppThemeMode? themeMode}) {
    return SettingsState(themeMode: themeMode ?? this.themeMode);
  }

  ThemeMode get flutterThemeMode {
    switch (themeMode) {
      case AppThemeMode.light:
        return ThemeMode.light;
      case AppThemeMode.dark:
        return ThemeMode.dark;
      case AppThemeMode.system:
        return ThemeMode.system;
    }
  }
}

class SettingsNotifier extends StateNotifier<SettingsState> {
  SettingsNotifier() : super(const SettingsState());

  void setThemeMode(AppThemeMode mode) {
    state = state.copyWith(themeMode: mode);
  }

  void cycleTheme() {
    final next = state.themeMode == AppThemeMode.light
        ? AppThemeMode.dark
        : AppThemeMode.light;
    state = state.copyWith(themeMode: next);
  }
}

final settingsProvider =
    StateNotifierProvider<SettingsNotifier, SettingsState>((ref) {
  return SettingsNotifier();
});
