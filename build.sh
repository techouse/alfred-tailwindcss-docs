#!/usr/bin/env sh
if [ -d "build/dist" ]; then
  rm -rf build/dist
fi

if [ -d "build/debug_info" ]; then
  rm -rf build/debug_info
fi

mkdir -p build/dist build/debug_info
cp -r info.plist assets/* LICENSE README.md build/dist
dart compile exe bin/main.dart -o build/dist/docs -S build/debug_info/docs
