#!/usr/bin/env bash

set -e

function main() {
  backend_pubkey=Arf1S41TvR4pG+i5FvrNFhzjiGqcavVFY5LBEBqgOExY
  admin=aura14mkz8a04hytpsmqt2znv0l8fmluvmdhfj98ds3
  operator=aura19fmuecv5gk2m5pmh5m3gg5g675fu8mnz6gwwug

  echo "admin: $admin"
  echo "operator: $operator"
  echo "backend_pubkey: $backend_pubkey"

  resolver_store_txhash=$(aurad tx wasm store artifacts/aurans_resolver.wasm --from admin --fees 25000ueaura --gas 5000000 --output json -y | jq -r '.txhash')
  sleep 10
  resolver_code_id=$(aurad query tx $resolver_store_txhash --output json | jq -r '.logs[0].events[-1].attributes[-1].value')
  echo "resolver_code_id: $resolver_code_id"

  name_store_txhash=$(aurad tx wasm store artifacts/aurans_name.wasm --from admin --fees 25000ueaura --gas 5000000 --output json -y | jq -r '.txhash')
  sleep 10
  name_code_id=$(aurad query tx $name_store_txhash --output json | jq -r '.logs[0].events[-1].attributes[-1].value')
  echo "name_code_id: $name_code_id"

  manager_store_txhash=$(aurad tx wasm store artifacts/aurans_manager.wasm --from admin --fees 25000ueaura --gas 5000000 --output json -y | jq -r '.txhash')
  sleep 10
  manager_code_id=$(aurad query tx $manager_store_txhash --output json | jq -r '.logs[0].events[-1].attributes[-1].value')
  echo "manager_code_id: $manager_code_id"

  init_manager="{\"admin\":\"$admin\",\"operator\":\"$operator\",\"max_year_register\":5,\"prices\":[[0,{\"denom\":\"ueaura\",\"amount\":\"100\"}],[1,{\"denom\":\"ueaura\",\"amount\":\"1000\"}],[2,{\"denom\":\"ueaura\",\"amount\":\"800\"}],[3,{\"denom\":\"ueaura\",\"amount\":\"500\"}],[4,{\"denom\":\"ueaura\",\"amount\":\"300\"}]],\"backend_pubkey\":\"$backend_pubkey\",\"name_code_id\":$name_code_id,\"resolver_code_id\":$resolver_code_id}"

  ins_manager_txhash=$(aurad tx wasm instantiate $manager_code_id "$init_manager" --label "aurans-manager" --from admin --fees 25000ueaura --gas 5000000 --output json -y --no-admin | jq -r '.txhash')
  sleep 10
  resolver_addr=$(aurad query tx $ins_manager_txhash --output json | jq -r '.logs[0].events[-3].attributes[-1].value')
  echo "resolver_addr: $resolver_addr"
  name_addr=$(aurad query tx $ins_manager_txhash --output json | jq -r '.logs[0].events[-1].attributes[-1].value')
  echo "name_addr: $name_addr"
  mamager_addr=$(aurad query wasm list-contract-by-code $manager_code_id --output json | jq -r '.contracts[0]')
  echo "manager_addr: $mamager_addr"
}

main
