import 'package:json_annotation/json_annotation.dart';

part 'search_result_hierarchy.g.dart';

@JsonSerializable()
class SearchResultHierarchy {
  const SearchResultHierarchy({
    required this.lvl0,
    this.lvl1,
    this.lvl2,
    this.lvl3,
    this.lvl4,
    this.lvl5,
    this.lvl6,
  });

  final String lvl0;
  final String? lvl1;
  final String? lvl2;
  final String? lvl3;
  final String? lvl4;
  final String? lvl5;
  final String? lvl6;

  String? getLevel(int level) {
    switch (level) {
      case 1:
        return lvl1;
      case 2:
        return lvl2;
      case 3:
        return lvl3;
      case 4:
        return lvl4;
      case 5:
        return lvl5;
      case 6:
        return lvl6;
      default:
        return lvl0;
    }
  }

  factory SearchResultHierarchy.fromJson(Map<String, dynamic> json) =>
      _$SearchResultHierarchyFromJson(json);

  Map<String, String?> toJson() => _$SearchResultHierarchyToJson(
    this,
  ).map((key, value) => MapEntry(key, value?.toString()));
}
