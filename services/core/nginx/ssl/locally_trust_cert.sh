#!/bin/bash

set -ex

sudo cp ./selfsigned.crt /etc/ca-certificates/trust-source/anchors/
sudo trust extract-compat
openssl verify -CAfile /etc/ssl/cert.pem ./selfsigned.crt
