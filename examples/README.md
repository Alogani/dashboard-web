# Dashboard Web Examples

This directory contains example configuration files and templates to help you set up and deploy the Dashboard Web application. These examples are provided as reference implementations that you can adapt to your specific environment.

## Configuration Files

### config.toml

The `config.toml` file is the main configuration file for the Dashboard Web application. It is read at startup and controls various aspects of the application's behavior.

Key features:
- Authentication and access control settings
- External link definitions
- Remote command execution configuration
- Logging and cookie settings

**Important notes:**
- Changes to this file require restarting the application, which will expire all authentication cookies
- If you only change passwords, you can send SIGHUP to reload users and passwords without a full restart
- The example provided is a comprehensive configuration similar to a production setup but tweaked for debugging

To use this configuration:
1. Copy it to your preferred location (default: `/etc/dashboard-web/config.toml` expected by init script)
2. Modify the settings to match your environment
3. Start the application with the path to your configuration file

### nginx-example

This directory contains example Nginx configuration files for setting up a reverse proxy with SSL for the Dashboard Web application.

Key features:
- Secure HTTPS configuration
- WebSocket support
- Proper header forwarding
- Virtual host configuration

To use these configurations:
1. Copy the relevant files to your Nginx configuration directory
2. Modify domain names, SSL certificate paths, and other settings to match your environment
3. Test and reload Nginx

### fail2ban.conf

The `fail2ban.conf` file provides configuration for integrating the Dashboard Web application with Fail2ban to protect against brute force attacks.

Key features:
- Filter definitions to detect authentication failures
- Jail configuration to block malicious IP addresses
- Customizable ban times and retry limits

To use this configuration:
1. Copy it to your Fail2ban configuration directory (typically `/etc/fail2ban/filter.d/`)
2. Create a jail configuration in `/etc/fail2ban/jail.d/` that references this filter
3. Adjust the settings to match your security requirements
4. Restart Fail2ban to apply the changes

## Deployment Scripts

The `package` folder in the project root contains several useful deployment scripts and configurations:

- `build-alpine.sh`: Script for building the application on Alpine Linux
- `install.sh`: Installation script that sets up the application and its dependencies. You can also run the binary directly.
- `openrc-init`: OpenRC init script for managing the application as a service

These scripts streamline the deployment process and provide a foundation for integrating the application into your infrastructure. They are particularly useful for production environments and automated deployments.

## Getting Started

To get started with these examples:

1. Review the example configuration files to understand the available options
2. Copy and adapt the relevant files to your environment
3. Follow the deployment instructions in the main README.md file

For more detailed information about specific configuration options, refer to the comments within each file or the documentation in the main project README.

## Additional Resources

- Main project documentation: See the README.md in the project root
- Configuration reference: See the comments in the example config.toml file
- Deployment guide: See the package/README.md file for detailed deployment instructions