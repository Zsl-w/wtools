import 'package:flutter_riverpod/flutter_riverpod.dart';

class SettingsState {
  const SettingsState();

  SettingsState copyWith() => const SettingsState();
}

class SettingsNotifier extends StateNotifier<SettingsState> {
  SettingsNotifier() : super(const SettingsState());
}

final settingsProvider =
    StateNotifierProvider<SettingsNotifier, SettingsState>((ref) {
  return SettingsNotifier();
});
