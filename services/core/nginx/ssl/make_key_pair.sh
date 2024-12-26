#!/bin/bash

# ECDSA with secp521r1 curve
openssl ecparam -name secp521r1 -genkey -noout -out selfsigned.key

# create certificate
openssl req -x509 -new -nodes -days 365 \
    -key ./selfsigned.key \
    -out ./selfsigned.crt \
    -config san.cnf \
    -extensions v3_req
