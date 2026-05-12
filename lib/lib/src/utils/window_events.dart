/// Callback invoked when the main window is shown (via hotkey or tray click).
void Function()? onWindowShown;

/// Counter to suppress window auto-hide while modal dialogs are open.
/// Incremented before opening a dialog (file picker, etc.), decremented after.
int _pickerCount = 0;

/// Whether any picker/modal dialog is currently open.
bool get isPickerOpen => _pickerCount > 0;

/// Call when opening a dialog that should suppress auto-hide.
void beginPicker() => _pickerCount++;

/// Call when closing a dialog that should suppress auto-hide.
void endPicker() => _pickerCount--;
