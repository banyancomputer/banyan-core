#!/bin/bash

set -o errexit

cargo sqlx prepare --database-url sqlite://$(pwd)/data/server.db -- --all-targets --all-features --tests
