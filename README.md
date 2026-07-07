# GnosisVPN Server

This binary aims to run alongside and manages a WireGuard server.

## Reporting Issues

We use **[Gnosis VPN](https://github.com/gnosis/gnosis_vpn)** repository as the
central hub for all user feedback.

### How to report an issue

1. Visit
   [Gnosis VPN Discussions](https://github.com/gnosis/gnosis_vpn/discussions)
   board.
1. Search existing Discussions and
   [Issues](https://github.com/gnosis/gnosis_vpn/issues) to see if your topic is
   already covered.
1. If not, start a new Discussion in the
   [Issues & Bug Reports](https://github.com/gnosis/gnosis_vpn/discussions/new?category=issues-bug-reports)
   category.
1. Provide as much detail as possible using the provided template.

The team will review all discussions and promote confirmed bugs or planned
features to actionable issues. This repository is reserved for tracking
actionable work on this component.

## Usage without GnosisVPN Client Communication

Sample configuration file for the current server side administration tasks:

```config.toml
allowed_client_ips = { start = "10.128.0.2", end = "10.128.0.100" }
wireguard_config_path = "/etc/wireguard/wg0.conf"
client_handshake_timeout_s = 300
```

The server private key will be taken from the specified WireGuard configuration file.
PostUp and PostDown hooks will be taken as is.

Sample wg0.conf:

```wg0.conf
[Interface]
Address = 10.128.0.1/9
PrivateKey = <somekey>
ListenPort = 51820
MTU = 1500
PostUp = iptables -t nat -A POSTROUTING -s 10.128.0.0/9 -o ens3 -j MASQUERADE
PostDown = iptables -t nat -D POSTROUTING -s 10.128.0.0/9 -o ens3 -j MASQUERADE
```

### User Statistics

Determine user statistics, run as root:

```bash
gnosis_vpn-server -c config.toml status --json
```

### Add New Client

Add a new client via client public key, run as root:

```bash
gnosis_vpn-server -c config.toml register <PUBLIC_KEY> --json --persist-config
```

## Usage with GnosisVPN Client Communication

Let the server manage wg clients without manual intervention.
This will periodically remove disconnected clients.
Meaning the ip range effectively determines the number of max concurrently connected clients.
Sample configuration file allowing **only** five connected clients:

```config.toml
allowed_client_ips = { start = "10.128.0.2", end = "10.128.0.6" }
wireguard_config_path = "/etc/wireguard/wg0.conf"
endpoint = "0.0.0.0:1429"
client_handshake_timeout_s = 300
client_cleanup_interval_s = 180
```

Run as an HTTP server:

```bash
gnosis_vpn-server -c config.toml serve --sync-wg-interface --periodically-run-cleanup
```

## Dependency Updates

Renovate runs on Renovate’s `schedule:earlyMondays` preset with a 14-day minimum release
age, so most PRs appear early Monday morning and only for packages that have been
released for at least two weeks.

Updates are grouped by ecosystem:

| Group               | What it covers                                        | Notes                                                                                                 |
| ------------------- | ----------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `nix flake updates` | `flake.lock` inputs (nixpkgs, crane, rust-overlay, …) | digest/pinDigest updates; `pinDigests` disabled for the `nix` manager since nix pins via `flake.lock` |
| `github-actions`    | `.github/workflows` action refs                       | digest-pinned                                                                                         |
| _(individual PRs)_  | Cargo crates                                          | one PR per crate                                                                                      |
| Docker images       | `docker/Dockerfile` base images                       | digest-pinned via global `pinDigests: true`                                                           |

`prCreation: immediate` is intentional — CI only triggers on pull request events, so
waiting for branch checks would deadlock.

## Deployment

Show potential deployment targets:

`nix flake show`

Build for a target, e.g. `x86_64-linux`:

`nix build .#gvpn-x86_64-linux`

The resulting binary is in `result/bin/`:

```
> ls -la result/bin/
total 6736
-r-xr-xr-x 1 root root 6886689  1. Jan 1970  gnosis_vpn-server
```
