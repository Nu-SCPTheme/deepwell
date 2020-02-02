#!/bin/bash
set -eu

if [[ $TRAVIS_OS_NAME != linux ]]; then
	echo 'Skipping database, only running on linux'
	exit 0
fi

case "$1" in
	setup)
		cargo install diesel_cli --no-default-features --features postgres
		psql -c "CREATE USER overseer PASSWORD 'blackmoon';" -U postgres
		psql -c "CREATE DATABASE deepwell OWNER overseer;" -U postgres
		;;
	check)
		diesel migration run
		diesel migration redo
		;;
	test)
		cargo test --release -- --nocapture
		;;
esac
