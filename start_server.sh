#!/bin/bash
set -eu

# Starts a local web-server that serves the contents of the `doc/` folder,
# which is the folder to where the web version is compiled.

cargo install basic-http-server

echo "open http://localhost:8080"

(cd docs && basic-http-server --addr 0.0.0.0:8080 .)
# (cd docs && python3 -m http.server 8080 --bind 0.0.0.0)
