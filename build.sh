#!/usr/bin/env bash

# Function to build header
oss_header() {
  first=$(printf '%.s#' {1..78})
  second=$(
    printf '##'
    printf '%.s ' {1..74}
    printf '##'
  )
  third=$(
    printf '##'
    printf '%.s ' {1..27}
    printf 'OPEN-SOURCE LICENSES'
    printf '%.s ' {1..27}
    printf '##'
  )
  printf "\n\n%s\n%s\n%s\n%s\n%s\n\n" "$first" "$second" "$third" "$second" "$first"
}

if [ -d "build/dist" ]; then
  rm -rf build/dist
fi

if [ -d "build/debug_info" ]; then
  rm -rf build/debug_info
fi

mkdir -p build/dist build/debug_info
cp -r info.plist assets/* LICENSE README.md build/dist

if command -v dart-pubspec-licenses-lite; then
  oss_header >>build/dist/LICENSE
  dart-pubspec-licenses-lite --pubspec-lock pubspec.lock >>build/dist/LICENSE
else
  echo 'Info: Unable to generate OSS LICENSES. Please install https://github.com/techouse/dart_pubspec_licenses_lite'
fi

dart compile exe bin/main.dart -o build/dist/docs -S build/debug_info/docs
