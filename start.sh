#!/bin/bash

docker-compose --env-file=./mongo.env up -d

sleep 5

docker exec mongo1 /scripts/rs-init.sh

sleep 20

docker exec mongo1 /scripts/index-init.sh
