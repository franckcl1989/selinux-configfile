# Security Policy

## Reporting a vulnerability

To report a security vulnerability, please open a private security advisory
on [GitHub](https://github.com/franck/selinux-configfile/security/advisories).

Please do not open a public issue for security vulnerabilities.

## Supported versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |

## Security considerations

This library reads and writes `/etc/selinux/config`, a system configuration
file that controls SELinux enforcement. Users should:

- Validate config values before writing to production systems
- Ensure the process has appropriate file permissions
- Use the atomic write API (`write_to`/`write_default`) to avoid corruption
