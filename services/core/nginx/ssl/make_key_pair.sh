#!/usr/bin/env bash

# ECDSA with prime256v1 curve (P-256)
openssl ecparam -name prime256v1 -genkey -noout -out selfsigned.key

# create certificate
openssl req -x509 -new -nodes -days 365 \
    -key ./selfsigned.key \
    -out ./selfsigned.crt \
    -config san.cnf \
    -extensions v3_req
