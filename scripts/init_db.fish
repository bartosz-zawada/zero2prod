#!/usr/bin/env fish

# set -l fish_trace on

function assert-sqlx-is-installed
    if not command -q sqlx
        echo >&2 "Error: sqlx is not installed"
        echo >&2 "Use:"
        echo >&2 "  cargo install sqlx-cli --no-default-features -F rustls,postgres"
        exit 1
    end
end

function set-with-default -d "Check if a var has been set, otherwise use default value"
    set -q $argv[1] || set -g $argv[1] $argv[2..-1]
end

function set-database-variables
    set-with-default DB_PORT 5432
    set-with-default SUPERUSER postgres
    set-with-default SUPERUSER_PWD password
    set-with-default APP_USER app
    set-with-default APP_USER_PWD secret
    set-with-default APP_DB_NAME newsletter
end

function docker-run -a container
    docker run \
        --env POSTGRES_USER=$SUPERUSER \
        --env POSTGRES_PASSWORD=$SUPERUSER_PWD \
        --health-cmd="pg_isready -U $SUPERUSER || exit 1" \
        --health-interval=1s \
        --health-timeout=5s \
        --health-retries=5 \
        --publish $DB_PORT:5432 \
        --detach \
        --name $container \
        postgres -N 1000
end

function docker-exists -a container
    [ (docker inspect --format "{{.State.Running}}" $container 2>/dev/null) ]
end

function docker-is-healthy -a container
    test (docker inspect --format "{{.State.Health.Status}}" $container 2>/dev/null) = healthy
end

function wait-until-healthy -a container
    set -l timeout 10
    set -l waited 0
    set -l period 1

    while not docker-is-healthy $container
        sleep $period
        set waited (math $waited + $period)
        if test $waited -ge $timeout
            echo >&2 "! $container container is still not healthy after $timeout seconds - exiting"
            exit 1
        end
    end

    echo >&2 "Postgres is up and running on port $DB_PORT!"
end

# Ensure sqlx is installed
assert-sqlx-is-installed

# Set the necessary env variables
set-database-variables

# Ensure the Postgres container is working
set -l container postgres

if test "$argv[1]" = recreate
    echo >&2 "> Database recreation enabled - Deleting $container container"
    docker rm -f $container
end

if not docker-is-healthy $container
    set -l fresh_container false
    if docker-exists $container
        echo >&2 "> $container container already exists - Starting"
        docker start $container
    else
        echo >&2 "> $container container does not exist - Creating"
        docker-run $container
        set fresh_container true
    end

    if not docker-is-healthy $container
        echo >&2 -e "\n> Waiting for $container container to be healthy"
        wait-until-healthy $container
    end

    if [ $fresh_container ]
        echo >&2 -e "\n> Creating user $APP_USER in database"
        set -l CREATE_QUERY "CREATE USER $APP_USER WITH PASSWORD '$APP_USER_PWD';"
        docker exec -it "$container" psql -U "$SUPERUSER" -c "$CREATE_QUERY"

        echo >&2 -e "\n> Providing user $APP_USER with CREATEDB privileges"
        set -l GRANT_QUERY "ALTER USER $APP_USER CREATEDB;"
        docker exec -it "$container" psql -U "$SUPERUSER" -c "$GRANT_QUERY"
    end

    echo >&2
end

# Set up the database
set -gx DATABASE_URL postgres://$APP_USER:$APP_USER_PWD@localhost:$DB_PORT/$APP_DB_NAME
echo >&2 -e "> Setting up database $APP_DB_NAME, and running migrations"
sqlx database setup

echo >&2 -e "\n> All done, you're ready to go!"
