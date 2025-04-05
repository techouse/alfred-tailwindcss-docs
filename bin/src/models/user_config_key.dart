import 'package:recase/recase.dart';

enum UserConfigKey {
  tailwindVersion,
  useAlfredCache,
  useFileCache,
  cacheTtl;

  @override
  String toString() => name.snakeCase;
}
