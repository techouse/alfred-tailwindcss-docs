import 'dart:io' show Platform, exitCode, stdout;

import 'package:alfred_workflow/alfred_workflow.dart'
    show
        AlfredItem,
        AlfredItemIcon,
        AlfredItemText,
        AlfredItems,
        AlfredWorkflow;
import 'package:algolia/algolia.dart' show AlgoliaQuerySnapshot;
import 'package:args/args.dart' show ArgParser, ArgResults;
import 'package:collection/collection.dart' show IterableExtension;
import 'package:html_unescape/html_unescape.dart' show HtmlUnescape;
import 'package:path/path.dart' show dirname;
import 'package:stash/stash_api.dart'
    show
        Cache,
        CacheEntryCreatedEvent,
        CacheEntryEvictedEvent,
        CacheEntryExpiredEvent,
        CacheEntryRemovedEvent,
        CacheEntryUpdatedEvent,
        CreatedExpiryPolicy,
        EventListenerMode,
        LruEvictionPolicy,
        CacheExtension;
import 'package:stash_file/stash_file.dart'
    show FileCacheStore, newFileLocalCacheStore;

import 'src/constants/config.dart' show Config;
import 'src/extensions/string_helpers.dart' show StringHelpers;
import 'src/models/search_result.dart' show SearchResult;
import 'src/services/algolia_search.dart' show AlgoliaSearch;

final HtmlUnescape unescape = HtmlUnescape();

final AlfredWorkflow workflow = AlfredWorkflow();

final FileCacheStore store = newFileLocalCacheStore(
  path: dirname(Platform.script.toFilePath()),
  fromEncodable: (Map<String, dynamic> json) => AlfredItems.fromJson(json),
);

final Cache cache = store.cache<AlfredItems>(
  name: 'query_cache',
  maxEntries: 10,
  eventListenerMode: EventListenerMode.synchronous,
  evictionPolicy: const LruEvictionPolicy(),
  expiryPolicy: const CreatedExpiryPolicy(Duration(minutes: 1)),
);

bool verbose = false;

void main(List<String> arguments) async {
  try {
    exitCode = 0;

    workflow.clearItems();

    final ArgParser parser = ArgParser()
      ..addOption('query', abbr: 'q', mandatory: true)
      ..addFlag('verbose', abbr: 'v', defaultsTo: false);
    final ArgResults args = parser.parse(arguments);

    verbose = args['verbose'];

    List<String> query =
        args['query'].replaceAll(RegExp(r'\s+'), ' ').trim().split(' ');
    String? version =
        query.firstWhereOrNull((el) => Config.supportedVersions.contains(el));
    if (version != null) {
      query.removeWhere((str) => str == version);
    } else {
      version = Config.supportedVersions.last;
    }
    final String queryString = query.join(' ').trim();

    if (verbose) {
      stdout.writeln('Query: "$queryString"');
      _cacheVerbosity();
    }

    if (queryString.isEmpty) {
      _showPlaceholder();
    } else {
      final AlfredItems? cachedItem =
          await cache.get('${queryString}_${version}'.md5hex);
      if (cachedItem != null) {
        workflow.addItems(cachedItem.items);
      } else {
        await _performSearch(queryString, version: version);
      }
    }
  } on FormatException catch (err) {
    exitCode = 2;
    workflow.addItem(AlfredItem(title: err.toString()));
  } catch (err) {
    exitCode = 1;
    workflow.addItem(AlfredItem(title: err.toString()));
    if (verbose) rethrow;
  } finally {
    workflow.run();
  }
}

void _cacheVerbosity() {
  cache
    ..on<CacheEntryCreatedEvent<AlfredItems>>()
        .listen((event) => print('Key "${event.entry.key}" added'))
    ..on<CacheEntryUpdatedEvent<AlfredItems>>()
        .listen((event) => print('Key "${event.newEntry.key}" updated'))
    ..on<CacheEntryRemovedEvent<AlfredItems>>()
        .listen((event) => print('Key "${event.entry.key}" removed'))
    ..on<CacheEntryExpiredEvent<AlfredItems>>()
        .listen((event) => print('Key "${event.entry.key}" expired'))
    ..on<CacheEntryEvictedEvent<AlfredItems>>()
        .listen((event) => print('Key "${event.entry.key}" evicted'));
}

void _showPlaceholder() {
  workflow.addItem(
    const AlfredItem(
      title: 'Search the Tailwind CSS docs...',
      icon: AlfredItemIcon(path: 'icon.png'),
    ),
  );
}

Future<void> _performSearch(String query, {String? version}) async {
  final AlgoliaQuerySnapshot snapshot = await AlgoliaSearch.query(
    query,
    version: version,
  );

  if (snapshot.nbHits > 0) {
    final AlfredItems items = AlfredItems(
      items: snapshot.hits
          .map((snapshot) => SearchResult.fromJson(snapshot.data))
          .map((result) {
        final int level = int.tryParse(result.type.substring(3)) ?? 0;
        final String? title = result.hierarchy.getLevel(level);
        final Map<String, String?> hierarchy = result.hierarchy.toJson()
          ..removeWhere((_, value) => value == null);

        return AlfredItem(
          uid: result.objectID,
          title: title!,
          subtitle:
              level > 0 ? unescape.convert(hierarchy.values.join(' > ')) : null,
          arg: result.url,
          text: AlfredItemText(
            largeType: title,
            copy: result.url,
          ),
          quickLookUrl: result.url,
          icon: AlfredItemIcon(path: 'icon.png'),
          valid: true,
        );
      }).toList(),
    );
    cache.putIfAbsent('${query}_${version}'.md5hex, items);
    workflow.addItems(items.items);
  } else {
    final Uri url =
        Uri.https('www.google.com', '/search', {'q': 'Tailwind CSS $query'});

    workflow.addItem(
      AlfredItem(
        title: 'No matching answers found',
        subtitle: 'Shall I try and search Google?',
        arg: url.toString(),
        text: AlfredItemText(
          copy: url.toString(),
        ),
        quickLookUrl: url.toString(),
        icon: AlfredItemIcon(path: 'google.png'),
        valid: true,
      ),
    );
  }
}
