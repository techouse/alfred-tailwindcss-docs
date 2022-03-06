#!/usr/bin/env sh
if [ -d "build/dist" ]; then
  rm -rf build/dist
fi
mkdir -p build/dist
cp -r info.plist assets/* LICENSE README.md version build/dist
dart compile exe bin/main.dart -o build/dist/docs -S /dev/null --verbosity error
