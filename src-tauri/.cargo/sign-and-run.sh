#!/usr/bin/env bash
# utility script for macOS dev that signs the binary before running it
# necessary because without entitlements, process memory r/w requires sudo
codesign -s - --entitlements "$(dirname "$0")/../Entitlements.plist" -f $1
$1
