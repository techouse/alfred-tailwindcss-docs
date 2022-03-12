import 'dart:convert' show utf8;

import 'package:crypto/crypto.dart' as crypto show md5;

extension StringHelpers on String {
  String get md5hex => crypto.md5.convert(utf8.encode(this)).toString();
}
