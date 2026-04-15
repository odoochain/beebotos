# WhatsApp Baileys Bridge for BeeBotOS

This Node.js bridge connects BeeBotOS to WhatsApp using the [Baileys](https://github.com/WhiskeySockets/Baileys) library.

## Features

- ✅ QR Code authentication
- ✅ Multi-device support
- ✅ Auto-reconnect
- ✅ Message deduplication
- ✅ Media download/upload (images, videos, audio, documents)
- ✅ Group support
- ✅ Location and contact messages
- ✅ Real-time message processing

## Prerequisites

- Node.js 16.0.0 or higher
- npm or yarn

## Installation

1. Navigate to the bridge directory:
```bash
cd tools/whatsapp-baileys-bridge
```

2. Install dependencies:
```bash
npm install
```

## Usage

### Standalone Mode

Run the bridge directly:

```bash
npm start
```

The bridge will:
1. Start and connect to WhatsApp Web
2. Display a QR code in the terminal
3. Wait for you to scan the QR code with your phone
4. Connect and start processing messages

### Integration with BeeBotOS

The bridge is automatically managed by the `WhatsAppAdapter` in Rust. When you start the WhatsApp processor:

1. The adapter spawns the Node.js bridge process
2. Communicates via stdio (JSON-RPC style)
3. Handles all WhatsApp operations

## Configuration

Configuration is passed via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `WHATSAPP_AUTH_DIR` | `./auth_info_baileys` | Directory for storing authentication |
| `WHATSAPP_MEDIA_DIR` | `./media/downloads` | Directory for downloaded media |
| `WHATSAPP_RECONNECT_INTERVAL` | `5000` | Reconnect interval in milliseconds |
| `WHATSAPP_MAX_RECONNECT_ATTEMPTS` | `10` | Maximum reconnection attempts |
| `WHATSAPP_LOG_LEVEL` | `info` | Log level (trace, debug, info, warn, error) |
| `WHATSAPP_PRINT_QR` | `true` | Whether to print QR code to terminal |

## Protocol

The bridge communicates with Rust via stdio using JSON messages:

### From Bridge to Rust (Events)

```json
{"type": "connected", "data": {"userId": "...", "userName": "..."}}
{"type": "qr", "data": {"qr": "..."}}
{"type": "message", "data": {...}}
{"type": "disconnected", "data": {"reason": "..."}}
```

### From Rust to Bridge (Commands)

```json
{"type": "send_text", "id": "uuid", "data": {"to": "1234567890@s.whatsapp.net", "text": "Hello"}}
{"type": "send_image", "id": "uuid", "data": {"to": "...", "imagePath": "...", "caption": "..."}}
{"type": "get_status", "id": "uuid", "data": {}}
```

## Message Types Supported

- **Text**: Plain text messages
- **Image**: Photos with optional caption
- **Video**: Videos with optional caption
- **Audio**: Audio files
- **Voice**: Voice messages (PTT)
- **Document**: Files (PDF, DOC, etc.)
- **Sticker**: Stickers
- **Location**: Shared locations
- **Contact**: Shared contacts

## Architecture

```
┌─────────────┐     stdio (JSON)     ┌─────────────────┐     WhatsApp Web
│  BeeBotOS   │ ◄──────────────────► │  Baileys Bridge │ ◄────────────────►
│   (Rust)    │                      │   (Node.js)     │    (WebSocket)
└─────────────┘                      └─────────────────┘
```

## Troubleshooting

### QR Code not displaying
- Check that `WHATSAPP_PRINT_QR=true`
- Ensure your terminal supports QR codes
- The QR code data is also sent to Rust for custom handling

### Connection issues
- Check your internet connection
- Ensure WhatsApp Web is not blocked
- Try clearing the auth directory: `rm -rf ./auth_info_baileys`

### Media not downloading
- Check `WHATSAPP_MEDIA_DIR` permissions
- Ensure sufficient disk space
- Check file size limits

## Security Notes

- The auth directory contains encrypted credentials - keep it secure
- Media files are stored locally - implement cleanup policies
- QR codes should be scanned immediately (they expire)
- Session data is persistent - delete auth directory to logout

## License

MIT
