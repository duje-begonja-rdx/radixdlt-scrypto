#!/bin/bash

set -x
set -e

resim="cargo run --manifest-path $PWD/simulator/Cargo.toml --bin resim $@ --"
scrypto_bindgen="cargo run --manifest-path $PWD/simulator/Cargo.toml --bin scrypto-bindgen $@ --"

file_contents="
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::consensus_manager::*;

use crate::prelude::*;

//==================================================================================================
// This file has been autogenerated by the ./update-bindings.sh script and none of the contents here
// are hand-written. If you make any changes to the interface of native blueprints and need to regen
// the bindings then run the update-bindings.sh script at the root of the repo.
//
// Note: there is currently no nice way to format this file since rustfmt doesn't format invocations
// of macros. So, while this is autogenerated, it comes at the price of the stubs readability.
//==================================================================================================

"

list=(
    "package_sim1pkgxxxxxxxxxfaucetxxxxxxxxx000034355863xxxxxxxxxhkrefh" # Faucet
    "package_sim1pkgxxxxxxxxxcnsmgrxxxxxxxxx000746305335xxxxxxxxxxc06cl" # Consensus Manager
    "package_sim1pkgxxxxxxxxxdntyxxxxxxxxxxx008560783089xxxxxxxxxnc59k6" # Identity
    "package_sim1pkgxxxxxxxxxaccntxxxxxxxxxx000929625493xxxxxxxxxrn8jm6" # Account
    "package_sim1pkgxxxxxxxxxplxxxxxxxxxxxxx020379220524xxxxxxxxxl5e8k6" # Pools
    "package_sim1pkgxxxxxxxxxcntrlrxxxxxxxxx000648572295xxxxxxxxxxc5z0l" # Access Controller
);
for address in ${list[@]}; 
do
    file_contents="$file_contents

$($scrypto_bindgen $address --reset-ledger)"
done

echo "$file_contents" > $PWD/scrypto/src/component/stubs.rs
rustfmt $PWD/scrypto/src/component/stubs.rs

python3 format-stubs.py