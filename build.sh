#!/usr/bin/env bash

docker build -t rust-analyzer .

# extract it
id=$(docker create rust-analyzer)
docker cp $id:/usr/local/bin/rust-analyzer ./
docker rm -v $id
