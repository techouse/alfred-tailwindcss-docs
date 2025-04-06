import 'package:recase/recase.dart';

enum UserConfigKey {
  tailwindVersion,
  useAlfredCache,
  useFileCache,
  cacheTtl,
  fileCacheMaxEntries;

  @override
  String toString() => name.snakeCase;
}
