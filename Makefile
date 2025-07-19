.PHONY: build submit results build-and-submit clean help

# Default target
help:
	@echo "Mwmbl Ranker - WASM Build and Submission"
	@echo ""
	@echo "Prerequisites:"
	@echo "  1. Register at https://mwmbl.org/accounts/signup/"
	@echo "  2. Set environment variables:"
	@echo "     export MWMBL_USERNAME=your_username"
	@echo "     export MWMBL_PASSWORD=your_password"
	@echo ""
	@echo "Available targets:"
	@echo "  build            - Compile Rust to WASM"
	@echo "  submit           - Submit WASM for evaluation"
	@echo "  results          - Check results for latest submission"
	@echo "  build-and-submit - Build and submit in one command"
	@echo "  clean            - Clean build artifacts"
	@echo "  help             - Show this help message"
	@echo ""
	@echo "Quick start:"
	@echo "  make build-and-submit"

# API configuration
API_BASE_URL = https://api.mwmbl.org/api/v1
PLATFORM_URL = $(API_BASE_URL)/platform
EVALUATE_URL = $(API_BASE_URL)/evaluate

# Temporary files
TOKEN_FILE = .tmp_token
JOB_ID_FILE = .tmp_job_id

# Check dependencies
check-deps:
	@echo "Checking dependencies..."
	@command -v wasm-pack >/dev/null 2>&1 || { echo "Error: wasm-pack not found. Install with: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"; exit 1; }
	@command -v curl >/dev/null 2>&1 || { echo "Error: curl not found. Please install curl."; exit 1; }
	@command -v jq >/dev/null 2>&1 || { echo "Error: jq not found. Please install jq for JSON parsing."; exit 1; }
	@echo "✓ All dependencies found"

# Check environment variables
check-env:
	@if [ -z "$(MWMBL_USERNAME)" ]; then \
		echo "Error: MWMBL_USERNAME environment variable not set"; \
		echo "Please set it with: export MWMBL_USERNAME=your_username"; \
		echo "If you don't have an account, register at https://mwmbl.org/accounts/signup/"; \
		exit 1; \
	fi
	@if [ -z "$(MWMBL_PASSWORD)" ]; then \
		echo "Error: MWMBL_PASSWORD environment variable not set"; \
		echo "Please set it with: export MWMBL_PASSWORD=your_password"; \
		exit 1; \
	fi

# Get JWT token
get-token: check-env
	@echo "Logging in to get JWT token..."
	@curl -s -X POST "$(PLATFORM_URL)/token/pair" \
		-H "Content-Type: application/json" \
		-d '{"username":"$(MWMBL_USERNAME)","password":"$(MWMBL_PASSWORD)"}' \
		| jq -r '.access' > $(TOKEN_FILE) 2>/dev/null || { \
			echo "Error: Login failed. Please check your credentials."; \
			echo "If you don't have an account, register at https://mwmbl.org/accounts/signup/"; \
			rm -f $(TOKEN_FILE); \
			exit 1; \
		}
	@if [ "$$(cat $(TOKEN_FILE))" = "null" ] || [ ! -s $(TOKEN_FILE) ]; then \
		echo "Error: Failed to get access token. Please check your credentials."; \
		echo "If you don't have an account, register at https://mwmbl.org/accounts/signup/"; \
		rm -f $(TOKEN_FILE); \
		exit 1; \
	fi
	@echo "✓ Successfully logged in"

# Build WASM
build: check-deps
	@echo "Building WASM package..."
	@wasm-pack build --target web --out-dir pkg
	@echo "✓ WASM build complete"

# Submit WASM for evaluation
submit: check-deps get-token
	@if [ ! -f "pkg/ranker_bg.wasm" ]; then \
		echo "Error: WASM file not found. Run 'make build' first."; \
		exit 1; \
	fi
	@echo "Submitting WASM for evaluation..."
	@TOKEN=$$(cat $(TOKEN_FILE)); \
	RESPONSE=$$(curl -s -X POST "$(EVALUATE_URL)/submit" \
		-H "Authorization: Bearer $$TOKEN" \
		-F "file=@pkg/ranker_bg.wasm"); \
	echo "$$RESPONSE" | jq -r '.job_id' > $(JOB_ID_FILE) 2>/dev/null || { \
		echo "Error: Submission failed"; \
		echo "Response: $$RESPONSE"; \
		rm -f $(TOKEN_FILE) $(JOB_ID_FILE); \
		exit 1; \
	}
	@JOB_ID=$$(cat $(JOB_ID_FILE)); \
	echo "✓ WASM submitted successfully"; \
	echo "Job ID: $$JOB_ID"; \
	echo "Run 'make results' to check evaluation status"
	@rm -f $(TOKEN_FILE)

# Check evaluation results
results: check-deps get-token
	@if [ ! -f "$(JOB_ID_FILE)" ]; then \
		echo "Error: No job ID found. Submit a WASM file first with 'make submit'."; \
		exit 1; \
	fi
	@echo "Checking evaluation results..."
	@TOKEN=$$(cat $(TOKEN_FILE)); \
	JOB_ID=$$(cat $(JOB_ID_FILE)); \
	RESPONSE=$$(curl -s -X GET "$(EVALUATE_URL)/results/$$JOB_ID" \
		-H "Authorization: Bearer $$TOKEN"); \
	echo "$$RESPONSE" | jq '.' 2>/dev/null || { \
		echo "Error: Failed to get results"; \
		echo "Response: $$RESPONSE"; \
		rm -f $(TOKEN_FILE); \
		exit 1; \
	}
	@rm -f $(TOKEN_FILE)

# Build and submit in one command
build-and-submit: build submit

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@rm -rf pkg target
	@rm -f $(TOKEN_FILE) $(JOB_ID_FILE)
	@echo "✓ Clean complete"
