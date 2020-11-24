#!/bin/bash

keyLength="2048"
days=3650
keyName="server.key"
certName="server.pem"


case `uname -s` in
    Linux*)  sslConfig=/etc/ssl/openssl.cnf;;
    Darwin*) sslConfig=/System/Library/OpenSSL/openssl.cnf;;
esac

openssl req \
    -newkey rsa:${keyLength} \
    -x509 \
    -nodes \
    -keyout ${keyName} \
    -new \
    -out ${certName} \
    -subj /CN=localhost \
    -reqexts SAN \
    -extensions SAN \
    -config <(cat $sslConfig <(printf '[SAN]\nsubjectAltName=DNS:localhost')) \
    -sha256 \
    -days ${days}