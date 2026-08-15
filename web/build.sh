#!/bin/bash
set -euo pipefail

env -u NO_COLOR trunk build
../target/debug/cli content diary
