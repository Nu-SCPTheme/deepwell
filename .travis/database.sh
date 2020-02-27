#!/bin/bash
set -eu

if [[ $TRAVIS_OS_NAME != linux ]]; then
	echo 'Skipping database, only running on linux'
	exit 0
fi

case "$1" in
	setup)
		# Setup postgres 12
		sudo sed -i 's/port = 5433/port = 5432/' /etc/postgresql/12/main/postgresql.conf
		sudo cp /etc/postgresql/{9.6,12}/main/pg_hba.conf
		sudo service postgresql stop
		sudo service postgresql start 12

		# Setup database
		cargo install diesel_cli --no-default-features --features postgres
		psql -c "CREATE USER overseer PASSWORD 'blackmoon';" -U postgres
		psql -c "CREATE DATABASE deepwell OWNER overseer;" -U postgres
		psql -c "CREATE DATABASE deepwell_test OWNER overseer;" -U postgres
		;;
	check)
		diesel migration run

		for stage in migrations/*; do
			diesel migration revert
		done

		diesel migration run
		;;
	test)
		cargo test --release -- --nocapture
		;;
esac
