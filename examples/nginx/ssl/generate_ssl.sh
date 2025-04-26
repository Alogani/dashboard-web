#!/bin/sh

# You must provide a cnf file
                             
# # Generate a proper ROOT CA   
# 1. Generate CA private key                
openssl genpkey -algorithm RSA \            
  -out nginx.key -pkeyopt rsa_keygen_bits:4096
                                            
# 2. Create self...signed CA cert (10 years)
openssl req -new -x509 -nodes \
  -key nginx.key -out nginx.crt \
  -days 3650 -sha256 \
  -config ca.cnf \ 
  -extensions v3_ca                      
                                         
# # nginxue the server certificate with SAN
# 1. Generate server private key               
openssl genpkey -algorithm RSA \               
  -out server.key -pkeyopt rsa_keygen_bits:2048
                  
# 2. CSR with SAN                  
openssl req -new \                 
  -key server.key -out server.csr \
  -config nginx.cnf \  
  -extensions req_ext                        
                                             
# 3. Sign CSR with proper CA extensions + SAN
openssl x509 -req \           
  -in server.csr \            
  -CA nginx.crt -CAkey nginx.key \
  -CAcreateserial \  
  -out server.crt \                   
  -days 365 -sha256 \                 
  -extfile nginx.cnf -extensions req_ext
                            
cp server.crt fullchain.pem 
cat nginx.crt >> fullchain.pem