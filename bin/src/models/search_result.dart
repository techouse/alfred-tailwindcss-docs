import 'package:json_annotation/json_annotation.dart';

import 'search_result_hierarchy.dart';

part 'search_result.g.dart';

@JsonSerializable(explicitToJson: true)
class SearchResult {
  const SearchResult({
    required this.objectID,
    required this.type,
    required this.url,
    required this.hierarchy,
    this.content,
  });

  final String objectID;
  final String type;
  final String url;
  final SearchResultHierarchy hierarchy;
  final String? content;

  static const List<String> attributesToRetrieve = [
    'hierarchy.lvl0',
    'hierarchy.lvl1',
    'hierarchy.lvl2',
    'hierarchy.lvl3',
    'hierarchy.lvl4',
    'hierarchy.lvl5',
    'hierarchy.lvl6',
    'content',
    'type',
    'url',
  ];

  static const List<String> attributesToSnippet = [
    'hierarchy.lvl1:10',
    'hierarchy.lvl2:10',
    'hierarchy.lvl3:10',
    'hierarchy.lvl4:10',
    'hierarchy.lvl5:10',
    'hierarchy.lvl6:10',
    'content:10',
  ];

  static const String snippetEllipsisText = '...';

  factory SearchResult.fromJson(Map<String, dynamic> json) =>
      _$SearchResultFromJson(json);

  Map<String, dynamic> toJson() => _$SearchResultToJson(this);
}
