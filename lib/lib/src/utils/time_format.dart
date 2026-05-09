/// Format a Unix timestamp (seconds) to a Chinese relative time string.
String formatRelativeTime(int timestampSec) {
  final now = DateTime.now();
  final time = DateTime.fromMillisecondsSinceEpoch(timestampSec * 1000);
  final diff = now.difference(time);

  if (diff.inSeconds < 60) return '刚刚';
  if (diff.inMinutes < 60) return '${diff.inMinutes} 分钟前';
  if (diff.inHours < 24) return '${diff.inHours} 小时前';
  if (diff.inDays < 7) return '${diff.inDays} 天前';

  return '${time.year}-${_pad(time.month)}-${_pad(time.day)} '
      '${_pad(time.hour)}:${_pad(time.minute)}';
}

String _pad(int n) => n.toString().padLeft(2, '0');
