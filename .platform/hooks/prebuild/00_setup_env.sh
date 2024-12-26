#!/bin/bash

# move the prod docker-compose file to the root for ebs to use

mv ./infra/prod.compose.yml ./docker-compose.yml
mv ./infra/env ./env
