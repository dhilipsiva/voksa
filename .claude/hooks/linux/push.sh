#!/bin/sh
. "$(dirname "$0")/env.sh"
git push origin HEAD --follow-tags
