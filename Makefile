.PHONY: build build-dashboard build-rust clean dev bundle install uninstall

VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')

# Build everything: dashboard frontend + Rust binary
build: build-dashboard build-rust

# Build the React dashboard
build-dashboard:
	cd dashboard && npm ci && npm run build

# Build the Rust binary (release mode)
build-rust:
	cargo build --release

# Build Rust in debug mode (faster iteration)
build-debug: build-dashboard
	cargo build

# Clean all build artifacts
clean:
	rm -rf dashboard/dist dist
	cargo clean

# Dev mode: run Express backend + Vite dev server (hot-reload)
dev:
	cd dashboard && npm run dev

# Run the embedded dashboard from Rust binary
run: build
	cargo run --release -- dashboard

# Create macOS .app bundle in dist/
bundle: build
	./packaging/macos/bundle.sh target/release/timely $(VERSION) dist

# Install Timely.app to /Applications and create CLI symlink
install: bundle
	@echo "Installing Timely.app to /Applications..."
	@if [ -d "/Applications/Timely.app" ]; then \
		rm -rf /Applications/Timely.app; \
	fi
	cp -R dist/Timely.app /Applications/Timely.app
	@echo "Creating symlink at /usr/local/bin/timely..."
	@ln -sf /Applications/Timely.app/Contents/MacOS/timely /usr/local/bin/timely
	@echo ""
	@echo "Timely installed successfully!"
	@echo "  App:     /Applications/Timely.app"
	@echo "  CLI:     /usr/local/bin/timely"
	@echo ""
	@echo "Next steps:"
	@echo "  1. Open System Settings > Privacy & Security > Accessibility"
	@echo "  2. Add Timely.app and grant permission"
	@echo "  3. Run: timely daemon start"

# Uninstall Timely
uninstall:
	@echo "Stopping daemon..."
	-launchctl remove com.timely.daemon 2>/dev/null
	@echo "Removing Timely.app..."
	rm -rf /Applications/Timely.app
	@echo "Removing CLI symlink..."
	rm -f /usr/local/bin/timely
	@echo "Timely uninstalled."
	@echo "Note: ~/.timely/ (data) was preserved. Remove manually if desired."
