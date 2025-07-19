## Ranking algorithms for the Mwmbl open source search engine

This repository contains Rust implementations of ranking algorithms for Mwmbl.
It is intended to replace the existing python implementation, and can
potentially be used from both Python and JavaScript code, via WebAssembly.

## Quick Start - Submit for Evaluation

To compile your ranking algorithm to WASM and submit it for evaluation:

### Prerequisites

1. **Register an account** at https://mwmbl.org/accounts/signup/
2. **Install dependencies:**
   - [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/): `curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`
   - [jq](https://jqlang.github.io/jq/download/): For JSON parsing
   - curl: Usually pre-installed on most systems

### Setup

1. **Set your credentials:**
   ```bash
   export MWMBL_USERNAME=your_username
   export MWMBL_PASSWORD=your_password
   ```
   
   Or copy and edit the example file:
   ```bash
   cp .env.example .env
   # Edit .env with your credentials
   source .env
   ```

### Usage

**Build and submit in one command:**
```bash
make build-and-submit
```

**Or step by step:**
```bash
# Compile Rust to WASM
make build

# Submit for evaluation
make submit

# Check evaluation results
make results
```

**Other commands:**
```bash
# Show help
make help

# Clean build artifacts
make clean
```

### What happens during submission

1. Your Rust code is compiled to WebAssembly using `wasm-pack`
2. The WASM file is submitted to the Mwmbl evaluation API
3. You receive a job ID to track the evaluation
4. Use `make results` to check the evaluation status and results

## Development

For local development and testing, see the `www/` directory which contains a webpack setup for testing your WASM module in a browser.
