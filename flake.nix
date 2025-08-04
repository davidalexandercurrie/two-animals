{
  description = "Two Animals - Rust-based game server";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain
            
            # Development tools
            cargo-watch
            just
            
            # For setup script
            curl
            
            # LLM tools
            claude-code
            ollama
            
            # Optional: for better development experience
            pkg-config
            openssl
          ];

          shellHook = ''
            echo "🎮 Two Animals development environment"
            echo "===================================="
            echo ""
            echo "Available LLM tools:"
            echo "  - claude-code: Claude CLI for AI assistance"
            echo "  - ollama: Local LLM server"
            echo ""
            
            # Check if .env exists
            if [ ! -f .env ]; then
              echo "⚠️  No .env file found!"
              echo ""
              echo "Quick setup:"
              echo "  For Ollama:  echo -e 'LLM_PROVIDER=ollama\nLLM_MODEL=llama3.2:latest' > .env"
              echo "  For Claude:  echo 'LLM_PROVIDER=claude' > .env"
              echo ""
            else
              echo "✅ .env file found"
              source .env
              echo "   Provider: $LLM_PROVIDER"
              if [ "$LLM_PROVIDER" = "ollama" ]; then
                echo "   Model: $LLM_MODEL"
              fi
              echo ""
            fi
            
            echo "Commands:"
            echo "  just dev     - Start the development server"
            echo "  ollama serve - Start Ollama server (run in separate terminal)"
            echo ""
            echo "Note: This flake allows unfree packages (required for claude-code)"
          '';

          # Set up environment variables for Rust
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };
      });
}
