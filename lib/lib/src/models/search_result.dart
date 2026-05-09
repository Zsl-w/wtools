/// Result model for the search UI — wraps both apps and files.
class SearchResult {
  final String id;
  final String name;
  final String path;
  final String type; // 'app' | 'file' | 'folder'
  final String? icon; // base64 data URI
  final List<String> aliases;
  final bool isRecent;
  final String? extension;
  final int? size; // bytes
  final String? modified;

  const SearchResult({
    required this.id,
    required this.name,
    required this.path,
    required this.type,
    this.icon,
    this.aliases = const [],
    this.isRecent = false,
    this.extension,
    this.size,
    this.modified,
  });

  factory SearchResult.fromApp(dynamic app) {
    return SearchResult(
      id: 'app:${app.path}',
      name: app.name,
      path: app.path,
      type: 'app',
      icon: app.icon,
      aliases: List<String>.from(app.aliases ?? []),
    );
  }

  factory SearchResult.fromFile(dynamic file) {
    final ext = file.name.contains('.')
        ? file.name.split('.').last.toLowerCase()
        : null;
    final isFolder = file.resultType == 'folder';
    return SearchResult(
      id: 'file:${file.path}',
      name: file.name,
      path: file.path,
      type: isFolder ? 'folder' : 'file',
      extension: ext,
      size: file.size is int ? file.size as int : null,
      modified: file.modified?.toString(),
    );
  }
}
