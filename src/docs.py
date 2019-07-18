#!/usr/bin/python
# encoding: utf-8

from __future__ import print_function, unicode_literals, absolute_import

import functools
import re
import sys
from collections import OrderedDict
from urllib import quote_plus

from algoliasearch.search_client import SearchClient
from config import Config
from workflow import Workflow, ICON_INFO

# Algolia client
client = SearchClient.create(Config.ALGOLIA_APP_ID, Config.ALGOLIA_SEARCH_ONLY_API_KEY)
index = client.init_index(Config.ALGOLIA_SEARCH_INDEX)

# log
log = None


def cache_key(query, version=Config.DEFAULT_TAILWIND_VERSION):
    """Make filesystem-friendly cache key"""
    key = "{}_{}".format(query, version)
    key = key.lower()
    key = re.sub(r"[^a-z0-9-_;.]", "-", key)
    key = re.sub(r"-+", "-", key)
    log.debug("Cache key : {!r} {!r} -> {!r}".format(query, version, key))
    return key


def handle_result(api_dict):
    """Extract relevant info from API result"""
    result = {}

    for key in {"objectID", "hierarchy", "content", "url", "anchor"}:
        if key == "hierarchy":
            api_dict[key] = OrderedDict(sorted(api_dict[key].items(), reverse=True))
            for hierarchy_key, hierarchy_value in api_dict[key].items():
                if hierarchy_value:
                    result["title"] = hierarchy_value
                    break
        else:
            result[key] = api_dict[key]

    return result


def search(
        query=None, version=Config.DEFAULT_TAILWIND_VERSION, limit=Config.RESULT_COUNT
):
    if query:
        results = index.search(
            query,
            {
                "facetFilters": ["version:v{}".format(version)],
                "page": 0,
                "hitsPerPage": limit,
            },
        )
        if results is not None and "hits" in results:
            return results["hits"]
    return []


def main(wf):
    if wf.update_available:
        # Add a notification to top of Script Filter results
        wf.add_item(
            "New version available",
            "Action this item to install the update",
            autocomplete="workflow:update",
            icon=ICON_INFO,
        )

    query = wf.args[0].strip()

    # Tag prefix only. Treat as blank query
    if query == "v":
        query = ""

    log.debug("query : {!r}".format(query))

    if not query:
        wf.add_item("Search the Tailwind CSS docs...")
        wf.send_feedback()
        return 0

    # Parse query into query string and tags
    words = query.split(" ")

    query = []
    version = Config.DEFAULT_TAILWIND_VERSION

    for word in words:
        if word in Config.SUPPORTED_TAILWIND_VERSIONS:
            version = word.replace("v", "")
        else:
            query.append(word)

    log.debug("version: " + version)
    log.debug("query without version: {!r}".format(query))

    query = " ".join(query)

    key = cache_key(query, version)

    results = [
        handle_result(result)
        for result in wf.cached_data(
            key, functools.partial(search, query, version), max_age=Config.CACHE_MAX_AGE
        )
    ]

    log.debug("{} results for {!r}, version {!r}".format(len(results), query, version))
    # Show results
    if not results:
        url = "https://www.google.com/search?q={}".format(quote_plus("Tailwind CSS {}".format(query)))
        wf.add_item(
            "No matching answers found", "Try a and search Google?",
            valid=True,
            arg=url,
            copytext=url,
            quicklookurl=url,
            icon=Config.GOOGLE_ICON,
        )

    for result in results:
        wf.add_item(
            uid=result["objectID"],
            title=result["title"],
            arg=result["url"],
            valid=True,
            largetext=result["title"],
            copytext=result["url"],
            quicklookurl=result["url"],
            icon=Config.TAILWIND_ICON,
        )
        # log.debug(result)

    wf.send_feedback()


if __name__ == "__main__":
    wf = Workflow(
        update_settings={
            "github_slug": "techouse/alfred-tailwindcss-docs",
            "frequency": 7,
        }
    )
    log = wf.logger
    sys.exit(wf.run(main))
