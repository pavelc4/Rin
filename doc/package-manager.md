# rpkg — Rin Package Manager

`rpkg` is a lightweight, pacman-style package manager for Rin Terminal. It leverages the [Termux](https://termux.dev/) package repository ecosystem directly from your device.

## Overview

rpkg synchronises with the Termux repository, resolves dependencies, and installs packages into the Rin prefix directory (`/data/data/com.rin/files`). It supports install, remove, search, query, and upgrade operations — all without root access.

> `rpkg` is available in PATH by default when you open Rin Terminal.

## Usage

### Package Database

| Command | Description |
|---------|-------------|
| `rpkg -Sy` | Sync package database from repository |

Run this first to fetch the latest package index.

### Installing Packages

| Command | Description |
|---------|-------------|
| `rpkg -S <package>` | Install a package (e.g., `rpkg -S vim`) |
| `rpkg -Sf <package>` | Force reinstall a package |

Installation resolves dependencies automatically and downloads packages from the Termux repository.

### Upgrading Packages

| Command | Description |
|---------|-------------|
| `rpkg -Su` | Upgrade all installed packages |
| `rpkg -Syu` | Sync database and upgrade all packages |

### Searching Packages

| Command | Description |
|---------|-------------|
| `rpkg -Ss <query>` | Search for packages (e.g., `rpkg -Ss python`) |

Searches package names and descriptions. Installed packages are marked with `[installed]`.

### Removing Packages

| Command | Description |
|---------|-------------|
| `rpkg -R <package>` | Remove/uninstall a package |

### Querying Installed Packages

| Command | Description |
|---------|-------------|
| `rpkg -Q` | List all installed packages |

## Examples

```bash
# Sync database first
rpkg -Sy

# Search for packages
rpkg -Ss nodejs
rpkg -Ss python

# Install packages
rpkg -S nodejs
rpkg -S python

# Upgrade everything
rpkg -Su

# List installed
rpkg -Q

# Remove a package
rpkg -R nodejs
```

## How It Works

rpkg runs as a Rust native library (`librpkg_cli.so`) symlinked to `$PREFIX/usr/bin/rpkg`. When you type `rpkg -S <package>` in the terminal:

1. The command is passed to the rpkg binary
2. It reads the package index (synced from the Termux repository)
3. Dependencies are resolved via a SAT-like resolver
4. Packages are downloaded and extracted to `$PREFIX`
5. Metadata is stored in a local SQLite database

## Paths

| Path | Purpose |
|------|---------|
| `/data/data/com.rin/files` | Rin prefix (`$PREFIX`) |
| `$PREFIX/usr/bin/` | Installed binaries |
| `$PREFIX/usr/lib/` | Shared libraries |
| `$PREFIX/var/lib/rpkg/` | Package database |
