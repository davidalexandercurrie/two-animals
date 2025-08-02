#!/bin/bash

# Script to set up LLM provider for Two Animals

echo "🎮 Two Animals - LLM Setup"
echo "=========================="
echo ""
echo "Available LLM providers:"
echo "  1) Claude (via CLI)"
echo "  2) Ollama (local models)"
echo ""
read -p "Select provider (1-2): " PROVIDER_CHOICE

case $PROVIDER_CHOICE in
    1)
        echo ""
        echo "📋 Setting up Claude CLI..."
        echo ""
        
        # Check if claude is installed
        if ! command -v claude &> /dev/null; then
            echo "❌ Claude CLI is not installed!"
            echo ""
            echo "Please install Claude CLI first."
            echo "Visit: https://docs.anthropic.com/en/docs/claude-cli"
            echo ""
            exit 1
        fi
        
        echo "✅ Claude CLI found!"
        echo ""
        
        # Create or update .env file
        if [ -f .env ]; then
            echo "Updating .env file..."
        else
            echo "Creating .env file..."
        fi
        cat > .env << EOF
# Two Animals LLM Configuration
LLM_PROVIDER=claude
EOF
        echo "✅ Configuration saved to .env"
        echo ""
        echo "To run the server with Claude:"
        echo "  just dev"
        echo ""
        echo "Make sure you have set up your Claude API key."
        ;;
        
    2)
        echo ""
        echo "📋 Setting up Ollama..."
        echo ""
        
        # Check if Ollama is already installed
        if command -v ollama &> /dev/null; then
            echo "✅ Ollama is already installed!"
            echo ""
            
            # Check if Ollama service is running
            if systemctl is-active --quiet ollama.service 2>/dev/null; then
                echo "✅ Ollama service is running"
            else
                echo "⚠️  Ollama is installed but the service is not running"
                echo ""
                echo "To start Ollama service:"
                echo "  sudo systemctl start ollama"
                echo ""
                echo "To enable Ollama to start automatically on boot:"
                echo "  sudo systemctl enable ollama"
            fi
        else
            echo "📥 Installing Ollama..."
            echo ""
            echo "This will install Ollama as a system service."
            echo "After installation:"
            echo "  - Ollama will run in the background as a service"
            echo "  - You can manage it with: sudo systemctl [start|stop|status] ollama"
            echo "  - To uninstall later: sudo systemctl stop ollama && sudo rm /usr/local/bin/ollama"
            echo ""
            read -p "Continue with installation? (y/n) " -n 1 -r
            echo
            
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                echo "Installation cancelled"
                exit 1
            fi
            
            # Install Ollama using their official script
            if ! curl -fsSL https://ollama.com/install.sh | sh; then
                echo "❌ Failed to install Ollama"
                exit 1
            fi
            
            echo ""
            echo "✅ Ollama installed successfully!"
        fi
        
        echo ""
        echo "📋 Ollama Service Management:"
        echo "  - Start service:    sudo systemctl start ollama"
        echo "  - Stop service:     sudo systemctl stop ollama"
        echo "  - Check status:     sudo systemctl status ollama"
        echo "  - View logs:        journalctl -u ollama -f"
        echo ""
        
        echo ""
        echo ""
        
        # List available models
        echo "📋 Available Ollama models:"
        if ollama list 2>/dev/null | grep -v "NAME" | grep -q .; then
            ollama list 2>/dev/null | grep -v "NAME" | awk '{print "  - " $1}'
        else
            echo "  (no models installed)"
        fi
        
        echo ""
        echo "Select a model for Two Animals:"
        echo ""
        echo "Popular models (January 2025):"
        echo "  1) llama3.2:latest (3B parameters, fast, good for quick responses)"
        echo "  2) llama3.3:latest (70B parameters, state-of-the-art, slower)"
        echo "  3) mistral:latest (7B parameters, good balance)"
        echo "  4) gemma2:2b (2B parameters, very fast, by Google)"
        echo "  5) deepseek-r1:7b (7B reasoning model, good for complex tasks)"
        echo "  6) phi-4:latest (14B parameters, by Microsoft)"
        echo "  7) Enter a custom model name"
        echo ""
        echo "🎯 Recommendation: If unsure, choose option 1 (llama3.2) for a good balance"
        echo "   of speed and quality. It's great for interactive games like this!"
        echo ""
        echo "💡 Tip: Visit https://ollama.com/library for the full list"
        echo ""
        
        read -p "Select model (1-7): " MODEL_SELECTION
        
        case $MODEL_SELECTION in
            1) MODEL_CHOICE="llama3.2:latest" ;;
            2) MODEL_CHOICE="llama3.3:latest" ;;
            3) MODEL_CHOICE="mistral:latest" ;;
            4) MODEL_CHOICE="gemma2:2b" ;;
            5) MODEL_CHOICE="deepseek-r1:7b" ;;
            6) MODEL_CHOICE="phi-4:latest" ;;
            7) 
                read -p "Enter model name: " MODEL_CHOICE
                ;;
            *)
                echo "Invalid selection. Please run the script again."
                exit 1
                ;;
        esac
        
        echo ""
        echo "Selected model: $MODEL_CHOICE"
        
        # Check if model needs to be pulled
        if ! ollama list 2>/dev/null | grep -q "^$MODEL_CHOICE"; then
            echo ""
            echo "📥 Model '$MODEL_CHOICE' not found locally."
            echo "Would you like to download it now? (y/n)"
            read -p "> " PULL_CHOICE
            
            if [ "$PULL_CHOICE" = "y" ] || [ "$PULL_CHOICE" = "Y" ]; then
                echo "Downloading $MODEL_CHOICE (this may take a few minutes)..."
                ollama pull $MODEL_CHOICE
            else
                echo "Please pull the model manually:"
                echo "  ollama pull $MODEL_CHOICE"
                exit 1
            fi
        fi
        
        echo ""
        echo "✅ Model '$MODEL_CHOICE' is ready!"
        echo ""
        
        # Create or update .env file
        if [ -f .env ]; then
            echo "Updating .env file..."
        else
            echo "Creating .env file..."
        fi
        cat > .env << EOF
# Two Animals LLM Configuration
LLM_PROVIDER=ollama
LLM_MODEL=$MODEL_CHOICE
EOF
        echo "✅ Configuration saved to .env"
        echo ""
        echo "To run Two Animals:"
        echo "  just dev"
        echo ""
        echo "⚠️  Important: Make sure Ollama service is running before starting the game:"
        echo "  sudo systemctl start ollama"
        echo ""
        echo "💡 Tip: If you don't want Ollama running all the time, you can:"
        echo "  - Start it only when playing: sudo systemctl start ollama"
        echo "  - Stop it when done: sudo systemctl stop ollama"
        echo "  - Disable auto-start on boot: sudo systemctl disable ollama"
        ;;
        
    *)
        echo "Invalid choice. Please run the script again."
        exit 1
        ;;
esac

echo ""
echo "🎯 Next steps:"
echo "  1. Run the server using the command shown above"
echo "  2. The server will start on http://localhost:3000"
echo "  3. Use the /turn/execute endpoint to run game turns"
echo ""
echo "Happy gaming! 🎮"