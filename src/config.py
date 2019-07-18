# encoding: utf-8


class Config(object):
    # Number of results to fetch from API
    RESULT_COUNT = 20
    # How long to cache results for
    CACHE_MAX_AGE = 20  # seconds
    # Icon
    TAILWIND_ICON = "icon.png"
    GOOGLE_ICON = "google.png"
    # supported docs
    SUPPORTED_TAILWIND_VERSIONS = {"v0", "v1"}
    DEFAULT_TAILWIND_VERSION = "1"
    # Algolia credentials
    ALGOLIA_APP_ID = "BH4D9OD16A"
    ALGOLIA_SEARCH_ONLY_API_KEY = "3df93446658cd9c4e314d4c02a052188"
    ALGOLIA_SEARCH_INDEX = "tailwindcss"
