# Wallet Mask

**Wallet Mask** is a privacy-focused Chrome extension that automatically detects and blurs cryptocurrency wallet addresses on web pages to prevent accidental exposure during screen sharing or streaming.

Powered by **Rust** and **WebAssembly (WASM)**, it offers high-performance pattern matching for EVM, Bitcoin, and Solana addresses.

## Features

- **Multi-Chain Support**:
  - **EVM (Ethereum, etc.)**: Full addresses (`0x...`) and ENS names (`*.eth`).
  - **Bitcoin (BTC)**: Legacy (`1...`, `3...`) and SegWit/Bech32 (`bc1...`).
  - **Solana (SOL)**: Base58 addresses.
- **Transaction Hash Detection**:
  - **ETH/EVM**: Transaction hashes (`0x` + 64 hex chars).
  - **Bitcoin**: Transaction IDs (64 hex chars).
  - **Solana**: Transaction signatures (86-88 Base58 chars).
- **Truncated Pattern Detection**: Handles shortened addresses and transaction hashes (e.g., `0x123...abc`) commonly found on explorers like Etherscan and Solscan.
- **Privacy First**: All processing happens locally in the browser. Zero data ever leaves your device.
- **Toggle Control**: One-click enable/disable via the popup menu.

## Architecture

This extension uses a hybrid architecture:
- **Rust/WASM (`crates/wasm-detector`)**: Handles all heavy regex pattern matching and validation logic. Compiled to WebAssembly for near-native performance.
- **JavaScript (`extension/content.js`)**: Manages DOM traversal, mutation observation, and applies the visual blur effect based on matches returned by WASM.

## Prerequisites

- **Node.js** & **npm** (for Prettier/utilities)
- **Rust** & **Cargo** (for WASM backend)
- **wasm-pack**: `cargo install wasm-pack`

## Development Setup

1.  **Clone the repository**:
    ```bash
    git clone <repository-url>
    cd wallet-mask
    ```

2.  **Build the WASM module**:
    ```bash
    cd crates/wasm-detector
    wasm-pack build --target web --out-dir ../../extension/pkg
    ```
    *Note: If you encounter `wasm-opt` errors, ensure it is disabled in `Cargo.toml` or installed.*

3.  **Load Extension in Chrome**:
    - Open `chrome://extensions/`
    - Enable **Developer mode** (top right).
    - Click **Load unpacked**.
    - Select the `extension/` directory.

## Project Structure

```
wallet-mask/
├── crates/
│   └── wasm-detector/      # Rust backend logic
│       ├── src/lib.rs      # Pattern matching & regex definitions
│       └── Cargo.toml      # Rust dependencies
├── extension/              # Chrome extension frontend
│   ├── manifest.json       # Extension configuration (Manifest V3)
│   ├── content.js          # DOM manipulation script
│   ├── background.js       # Service worker (if applicable)
│   ├── icons/              # Extension icons
│   └── pkg/                # Compiled WASM output (auto-generated)
└── README.md               # You are here
```