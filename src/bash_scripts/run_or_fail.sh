# !/bin/bash

function run_or_fail {
    first_arg = $1
    shift
    $@
  if [ $? != 0 ]; then
    echo $first_arg
    exit 1
  fi
}