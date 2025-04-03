#!/bin/sh
set -euf

npx next build && cp _redirects out/_redirects
