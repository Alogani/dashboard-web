This is an example of configuration close to mine. 
It might be incomplete and must be adapted to needs.

No configuration has been audited, use with care and knowledge. I take no responsability

# Nginx Configuration Example for Dashboard Web

This directory contains example Nginx configurations for deploying the Dashboard Web application in a production environment. These configurations demonstrate how to set up Nginx as a reverse proxy for dashboard-web.

## Contents

- `http.d/` - Directory containing configuration files:
  - `nginx.conf` - Contains host mapping configurations
  - `root.conf` - Main server configuration for nginx.lan
  - `server_common.include` - Common server settings included by other configs
  - `auth.include` - Authentication configuration
  - `subdomains.conf` - Configuration for handling subdomains

## Key Features

- SSL/TLS configuration with recommended security settings
- Reverse proxy setup for Axum web services
- Rate limiting to protect against abuse
- HTTP to HTTPS redirection
- Static file serving configuration

## Configuration Overview

In these examples, dashboard-web is served on 127.0.0.1:8080 and acts as both the main application and the authentication service.

### nginx.conf
Defines upstream mappings for different subdomains and extracts subdomain information from the host header.

### root.conf
Configures the main server (nginx.lan) to proxy requests to dashboard-web.

### server_common.include
Contains common SSL and proxy settings shared across server blocks.

### auth.include
Implements authentication via auth_request module, directing authentication checks to dashboard-web's auth service.

### subdomains.conf
Handles routing for various subdomains, with special configuration for services like incus that require client certificates.

## Installation

1. Install Nginx on your server:

```bash
# For Debian/Ubuntu
sudo apt update
sudo apt install nginx

# For Alpine Linux
apk add nginx
```
