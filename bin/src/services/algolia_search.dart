import 'package:algoliasearch/algoliasearch_lite.dart';

import '../env/env.dart';
import '../models/search_result.dart';

class AlgoliaSearch {
  AlgoliaSearch._();

  static final SearchClient _client = SearchClient(
    appId: Env.algoliaApplicationId,
    apiKey: Env.algoliaSearchOnlyApiKey,
  );

  static Future<SearchResponse> query(String queryString, {String? version}) =>
      _client.searchIndex(
        request: SearchForHits(
          indexName: Env.algoliaSearchIndex,
          query: queryString,
          facetFilters: ['version:${version ?? Env.supportedVersions.last}'],
          attributesToRetrieve: SearchResult.attributesToRetrieve,
          attributesToSnippet: SearchResult.attributesToSnippet,
          snippetEllipsisText: SearchResult.snippetEllipsisText,
          distinct: 1,
          page: 0,
          hitsPerPage: 20,
        ),
      );

  static dispose() => _client.dispose();
}
