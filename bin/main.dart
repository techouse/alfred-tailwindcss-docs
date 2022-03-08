import 'dart:io' show exitCode, stdout;

import 'package:alfred_workflow/alfred_workflow.dart'
    show AlfredItem, AlfredItemIcon, AlfredItemText, AlfredWorkflow;
import 'package:algolia/algolia.dart' show AlgoliaQuerySnapshot;
import 'package:args/args.dart' show ArgParser, ArgResults;
import 'package:collection/collection.dart' show IterableExtension;
import 'package:easy_debounce/easy_debounce.dart' show EasyDebounce;
import 'package:html_unescape/html_unescape.dart' show HtmlUnescape;

import 'src/constants/config.dart' show Config;
import 'src/models/search_result.dart' show SearchResult;
import 'src/services/algolia_search.dart' show AlgoliaSearch;

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
    final HtmlUnescape unescape = HtmlUnescape();

    workflow.addItems(
      snapshot.hits.map((snapshot) => SearchResult.fromJson(snapshot.data)).map(
        (result) {
          final int level = int.tryParse(result.type.substring(3)) ?? 0;
          final String? title = result.hierarchy.getLevel(level);
          final Map<String, String?> hierarchy = result.hierarchy.toJson()
            ..removeWhere((_, value) => value == null);

          return AlfredItem(
            uid: result.objectID,
            title: title!,
            subtitle: level > 0
                ? unescape.convert(hierarchy.values.join(' > '))
                : null,
            arg: result.url,
            text: AlfredItemText(
              largeType: title,
              copy: result.url,
            ),
            quickLookUrl: result.url,
            icon: AlfredItemIcon(path: 'icon.png'),
            valid: true,
          );
        },
      ),
    );
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

final AlfredWorkflow workflow = AlfredWorkflow();
bool verbose = false;

void main(List<String> arguments) async {
  try {
    exitCode = 0;

    workflow.clearItems();

    final ArgParser parser = ArgParser()
      ..addOption('query', abbr: 'q', mandatory: true)
      ..addFlag('verbose', abbr: 'v', defaultsTo: false);
    final ArgResults args = parser.parse(arguments);

    List<String> query =
    args['query'].replaceAll(RegExp(r'\s+'), ' ').trim().split(' ');
    String? version = query.firstWhereOrNull(
          (el) => Config.supportedVersions.contains(el),
    );
    if (version != null) {
      query.removeWhere((str) => str == version);
    } else {
      version = Config.supportedVersions.last;
    }
    final String queryString = query.join(' ').trim();

    if (args['verbose']) verbose = true;

    if (verbose) stdout.writeln('Query: "$queryString"');

    EasyDebounce.debounce(
      'search',
      Duration(milliseconds: 250),
          () async {
        if (queryString.isEmpty) {
          _showPlaceholder();
        } else {
          await _performSearch(
            queryString,
            version: version,
          );
        }

        workflow.run();
      },
    );
  } on FormatException catch (err) {
    exitCode = 2;
    workflow.addItem(AlfredItem(title: err.toString()));
    workflow.run();
  } catch (err) {
    exitCode = 1;
    workflow.addItem(AlfredItem(title: err.toString()));
    workflow.run();
    if (verbose) {
      rethrow;
    }
  }
}
