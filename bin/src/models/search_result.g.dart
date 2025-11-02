// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'search_result.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

SearchResult _$SearchResultFromJson(Map<String, dynamic> json) => SearchResult(
  objectID: json['objectID'] as String,
  type: json['type'] as String,
  url: json['url'] as String,
  hierarchy: SearchResultHierarchy.fromJson(
    json['hierarchy'] as Map<String, dynamic>,
  ),
  content: json['content'] as String?,
);

Map<String, dynamic> _$SearchResultToJson(SearchResult instance) =>
    <String, dynamic>{
      'objectID': instance.objectID,
      'type': instance.type,
      'url': instance.url,
      'hierarchy': instance.hierarchy.toJson(),
      'content': instance.content,
    };
