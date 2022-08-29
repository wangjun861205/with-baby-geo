#!/bin/bash

docker-compose --env-file=./mongo.env up -d --scale with-baby-geo=3

sleep 5

docker exec mongo1 /scripts/rs-init.sh