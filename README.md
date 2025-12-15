# Shy to Text

A lightweight, "privacy-focused" speech-to-text application that runs entirely on your device using local Whisper AI models. No internet connection required for transcription.

## Features

- **Local AI Processing**: Uses Whisper models locally for complete privacy
- **Real-time Transcription**: Record and transcribe speech instantly
- **Multi-language Support**: Supports any language supported by the model of your choosing
- **Global Hotkeys**: Press F9 (customizable) to start/stop recording from anywhere
- **Tray Integration**: Minimizes to system tray for unobtrusive operation
- **Auto Copy**: Automatically copies transcriptions to clipboard
- **Customizable Settings**: Configure language, hotkey, and behavior preferences
- **Cross-platform**: Available for Linux and Windows

## Installation

### Prerequisites

- [Node.js](https://nodejs.org/) (v16 or higher)
- [Rust](https://rustup.rs/) (latest stable)
- [Bun](https://bun.sh/) (recommended for development)

### Download Whisper Models

Before using the app, you need to download Whisper model files:

1. Visit the [Hugging Face Whisper models page](https://huggingface.co/ggerganov/whisper.cpp/tree/main)
2. Download one of the `.bin` model files (recommended: `ggml-small-q8_0.bin`)
3. Click on the "choose" button inside the application to select the model

### Building from Source

```sh
git clone https://github.com/Brodino/shy-to-text
cd shy-to-text
bun install
bun run tauri build
```

The built application will be in `src-tauri/target/release/bundle/`.

### Development

```bash
# Start development server
bun run tauri dev
```

### Todo
- An actual logo
- Hardware acceleration
- Better settings management
