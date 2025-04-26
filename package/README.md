# Package Directory

This directory contains scripts and configuration files for building, deploying, and configuring the application.

## Contents

### Build Scripts

- **build-alpine.sh**: Shell script for building the application in an Alpine Linux environment. This script is optimized for creating lightweight Docker containers.

### Configuration Files

- **config.toml**: Main configuration file for the application. Contains settings for database connections, server parameters, and other application-specific configurations.

### Installation Scripts

- **install.sh**: Main installation script that sets up the application environment, installs dependencies, and configures the application based on the provided configuration files.

### Service Scripts

- **dashboard-web-init-script**: Initialization script for the dashboard web service. This script is used to start the web dashboard component of the application as a system service.

## Usage

### Building for Alpine Linux

To build the application for Alpine Linux (useful for Docker containers):

```bash
./build-alpine.sh
```

## Installing the Application

To install the application:

  `./install.sh`

This script will:
1. Set up the necessary environment
2. Install required dependencies
3. Configure the application using config.toml

## Configuring the Application

Edit config.toml to customize application settings before installation or to update settings after installation.

As a safety, missing or mispelled arguments will prevent the app to start.
However, typo in usernames for the access_rules are not currently checks

# Notes

* Make sure to review and modify configuration files before running the installation script
* The build and installation scripts require root privileges depending on your system configuration
* Always back up your configuration files before making changes
