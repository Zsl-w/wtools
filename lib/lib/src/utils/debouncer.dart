import 'dart:async';

/// A simple debouncer that delays calling [action]
/// until [duration] has passed since the last call.
class Debouncer {
  final Duration duration;
  Timer? _timer;

  Debouncer(this.duration);

  void run(void Function() action) {
    _timer?.cancel();
    _timer = Timer(duration, action);
  }

  void cancel() {
    _timer?.cancel();
    _timer = null;
  }
}
