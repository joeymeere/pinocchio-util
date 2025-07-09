#!/usr/bin/env bash

set -ex

workspace_crates=(
    pinocchio-util
    pinocchio-account-util
)

for crate in "${workspace_crates[@]}"; do
   echo "--- $crate"
   cargo package -p $crate
   cargo publish -p $crate
done