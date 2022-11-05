import 'package:envied/envied.dart';

part 'env.g.dart';

@Envied(path: '.env')
abstract class Env {
  @EnviedField(varName: 'APP_VERSION')
  static const String appVersion = _Env.appVersion;

  @EnviedField(varName: 'GITHUB_REPOSITORY_URL')
  static const String githubRepositoryUrl = _Env.githubRepositoryUrl;

  @EnviedField(varName: 'ALGOLIA_APPLICATION_ID', obfuscate: true)
  static final String algoliaApplicationId = _Env.algoliaApplicationId;

  @EnviedField(varName: 'ALGOLIA_SEARCH_ONLY_API_KEY', obfuscate: true)
  static final String algoliaSearchOnlyApiKey = _Env.algoliaSearchOnlyApiKey;

  @EnviedField(varName: 'ALGOLIA_SEARCH_INDEX', obfuscate: true)
  static final String algoliaSearchIndex = _Env.algoliaSearchIndex;

  @EnviedField(varName: 'SUPPORTED_VERSIONS')
  static const String _supportedVersions = _Env._supportedVersions;

  static final List<String> supportedVersions = _supportedVersions.split(',');
}
