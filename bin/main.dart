import 'dart:io' show exitCode, stdout;

import 'package:alfred_workflow/alfred_workflow.dart'
    show
        AlfredItem,
        AlfredItemIcon,
        AlfredItemText,
        AlfredItems,
        AlfredUpdater,
        AlfredWorkflow;
import 'package:algolia/algolia.dart' show AlgoliaQuerySnapshot;
import 'package:args/args.dart' show ArgParser, ArgResults;
import 'package:collection/collection.dart' show IterableExtension;
import 'package:html_unescape/html_unescape.dart' show HtmlUnescape;

import 'src/env/env.dart' show Env;
import 'src/models/search_result.dart' show SearchResult;
import 'src/services/algolia_search.dart' show AlgoliaSearch;

part 'main_helpers.dart';

bool _verbose = false;
bool _update = false;

void main(List<String> arguments) async {
  try {
    exitCode = 0;

    _workflow.clearItems();

    final ArgParser parser = ArgParser()
      ..addOption('query', abbr: 'q', defaultsTo: '')
      ..addFlag('verbose', abbr: 'v', defaultsTo: false)
      ..addFlag('update', abbr: 'u', defaultsTo: false);
    final ArgResults args = parser.parse(arguments);

    _update = args['update'];
    if (_update) {
      stdout.writeln('Updating workflow...');

      return await _updater.update();
    }

    _verbose = args['verbose'];

    List<String> query =
        args['query'].replaceAll(RegExp(r'\s+'), ' ').trim().split(' ');
    String? version =
        query.firstWhereOrNull((el) => Env.supportedVersions.contains(el));
    if (version != null) {
      query.removeWhere((str) => str == version);
    } else {
      version = Env.supportedVersions.last;
    }
    final String queryString = query.join(' ').trim().toLowerCase();

    if (_verbose) stdout.writeln('Query: "$queryString"');

    if (queryString.isEmpty) {
      _showPlaceholder();
    } else {
      _workflow.cacheKey = '${queryString}_$version';
      if (await _workflow.getItems() == null) {
        await _performSearch(queryString, version: version);
      }
    }
  } on FormatException catch (err) {
    exitCode = 2;
    _workflow.addItem(AlfredItem(title: err.toString()));
  } catch (err) {
    exitCode = 1;
    _workflow.addItem(AlfredItem(title: err.toString()));
    if (_verbose) rethrow;
  } finally {
    if (!_update) {
      if (await _updater.updateAvailable()) {
        _workflow.run(addToBeginning: updateItem);
      } else {
        _workflow.run();
      }
    }
  }
}
