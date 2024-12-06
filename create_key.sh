# Create private key
openssl genrsa -out developer.key 2048

# Create a configuration file for the certificate
echo "
[req]
distinguished_name = req_distinguished_name
x509_extensions = v3_req
prompt = no

[req_distinguished_name]
CN = Developer Code Signing
O = Development
C = US

[v3_req]
basicConstraints = critical,CA:FALSE
keyUsage = critical,digitalSignature
extendedKeyUsage = critical,codeSigning
" > developer.conf

# Create and sign certificate
openssl req -new -x509 -days 365 -key developer.key -out developer.crt -config developer.conf

# Unlock keychain
security unlock-keychain -p temppass Development.keychain

# Import certificate and key with more specific parameters
security import developer.crt -k Development.keychain -T /usr/bin/codesign \
    -x -P "" -A -t cert -f pkcs12
security import developer.key -k Development.keychain -T /usr/bin/codesign \
    -x -P "" -A -t priv -f pkcs12

# Set trust settings
sudo security add-trusted-cert -d -k Development.keychain -r trustRoot developer.crt