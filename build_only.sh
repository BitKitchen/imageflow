#!/bin/bash
set -e #Exit on failure.

TEST_C=False TEST_RUST=False VALGRIND=False ./build.sh
