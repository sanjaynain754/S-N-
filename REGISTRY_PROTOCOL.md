# S+N++ Remote Registry Protocol v1

## Design goals

The registry protocol is versioned, deterministic and transport-neutral. A client must be able to list versions, retrieve package metadata, download an archive and verify its contents without executing registry-provided instructions. The filesystem registry fixture and the future HTTP implementation use the same logical package identity: `name` plus exact semantic `version`.

## Base URL and media types

A registry is configured by a base URL such as `https://registry.example.org`. The protocol prefix is `/v1`. Clients send the user-agent `snp/snp-registry-v1` and should advertise the following media types:

| Resource | Media type |
|---|---|
| Package index | `application/vnd.snp.registry.index+json;v=1` |
| Package metadata | `application/vnd.snp.package.metadata+toml;v=1` |
| Package archive | `application/octet-stream` |

## Endpoints

| Method | Endpoint | Purpose |
|---|---|---|
| `GET` | `/v1/index/{package}.json` | List all published versions and their dependency metadata |
| `GET` | `/v1/packages/{package}/{version}/snp.toml` | Retrieve the canonical package manifest |
| `GET` | `/v1/packages/{package}/{version}/download` | Download the package archive |

Package names and versions must be URL-path safe. The client must reject a response whose package name or exact version does not match the request. Redirects should be limited to HTTPS locations under the configured registry origin.

## Index document

The index response is JSON with this shape:

```json
{
  "protocol": "snp-registry-v1",
  "package": "net.http",
  "versions": [
    {
      "version": "1.2.0",
      "metadata_url": "https://registry.example.org/v1/packages/net.http/1.2.0/snp.toml",
      "archive_url": "https://registry.example.org/v1/packages/net.http/1.2.0/download",
      "checksum": "sha256:...",
      "dependencies": [
        { "name": "std.net", "version": "^1.0.0" }
      ]
    }
  ]
}
```

`checksum` covers the exact downloaded archive bytes and uses the format `sha256:<lowercase-hex>`. Dependency constraints use the S+N++ semantic-version grammar. Versions must be sorted by the client before selecting the highest version that satisfies the requested constraint; server ordering is not trusted.

## Retrieval and integrity flow

The client first fetches and validates the index, chooses a compatible `IndexVersion`, fetches `snp.toml`, then downloads the archive. It verifies the archive checksum before placing it in the cache. A failed checksum, invalid semantic version, mismatched package identity, unsupported protocol, malformed JSON or unexpected HTTP status is a hard resolution error.

The cache key is `{registry-origin}/{package}/{version}/{checksum}`. A cached archive may be reused only when its stored checksum equals the index checksum. Metadata and archives are immutable after publication; a changed checksum for an existing exact package version is treated as a registry integrity error.

## HTTP error model

The client reports structured categories rather than exposing raw transport text as a resolver decision: `RegistryUnavailable`, `RegistryHttpStatus`, `RegistryProtocol`, `RegistryMalformedIndex`, `RegistryPackageMismatch`, `RegistryChecksumMismatch` and `RegistryVersionUnavailable`. Retries are allowed for network failures and HTTP 5xx responses, but not for 4xx responses or integrity failures.

## Security boundary

Archives are data only. The client must validate archive paths before extraction, reject absolute paths and `..` traversal, enforce a maximum archive size, and never run build scripts from a package. Registry authentication and signed metadata are reserved for a later protocol revision; HTTPS and archive checksums are required for the initial implementation.
