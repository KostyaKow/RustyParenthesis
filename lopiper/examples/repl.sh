#!/bin/bash

#evu=phantomjs
#evu="node -i"
evu=./scheme-electron/run.py
#(scm.screenshot "http://github.com" "test.png")

./_repl_helper.sh | $evu
