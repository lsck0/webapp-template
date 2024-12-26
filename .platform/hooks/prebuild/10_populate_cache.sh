#!/bin/bash

# aws elastic beanstalk has a maximum allowed build time of 5 minutes, this gets around that.

docker compose build
