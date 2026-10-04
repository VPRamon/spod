# Security policy

## Reporting a vulnerability

Please report security vulnerabilities privately using a GitHub private security advisory for this repository or by emailing `security@siderust.org`.

Do **not** open a public issue or pull request for a vulnerability.

## Supported versions

Until `spod` reaches 1.0, security fixes target the latest state of the `main` branch.

## Scope

Security-sensitive areas include:

- the `spod` CLI and experimental REST server;
- parsers for externally supplied files such as SP3, RINEX, ANTEX, CRD, CPF, EOP and YAML configuration;
- deserialisation and file-system entry points reachable through the service layer.

Vulnerabilities in the foundational Siderust crates should be reported in the corresponding upstream repository.

## REST hardening status

The REST interface is experimental and currently unauthenticated. Bind it only to trusted/local interfaces and do not expose it directly to an untrusted network.
