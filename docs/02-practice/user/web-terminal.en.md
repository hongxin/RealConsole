# Web Terminal

**[中文](web-terminal.md)** | English

> New feature in v1.23.0

RealConsole supports a browser-accessible Web Terminal, allowing remote access to RealConsole within a local network.

## Quick Start

### Prerequisites

1. **Configure API Key** (for LLM conversations)

```bash
# Set Deepseek API Key
export DEEPSEEK_API_KEY="your-api-key-here"
```

2. **Create configuration file** `realconsole.yaml` (optional, defaults work too)

```yaml
llm:
  primary:
    provider: "deepseek"
    model: "deepseek-chat"
    api_key: "${DEEPSEEK_API_KEY}"
```

### Start Web Service

#### Local Access (Default, Recommended)

```bash
# Start with default settings (127.0.0.1:7788)
realconsole web

# Custom port
realconsole web --port 9000
```

**Access URL**: http://127.0.0.1:7788

**Features**:
- Only accessible locally, most secure
- Suitable for personal development and testing
- Default configuration, no extra parameters needed

#### LAN Access (Use with Caution)

```bash
# Bind to all network interfaces (allows LAN access)
realconsole web --bind 0.0.0.0

# Bind to specific IP (recommended, more secure)
realconsole web --bind 192.168.1.100

# Custom port
realconsole web --bind 0.0.0.0 --port 9000
```

**Access URLs**:
- Local: http://127.0.0.1:7788
- LAN: http://\<your-IP\>:7788

**Get your IP**:
```bash
# macOS / Linux
ifconfig | grep "inet " | grep -v 127.0.0.1
```

**Security Warning**:
- 0.0.0.0 exposes the service to the entire LAN
- Current version has no authentication
- Only use in trusted networks
- Has full Shell execution permissions

## Configuration File

Add Web configuration to `realconsole.yaml`:

```yaml
# LLM configuration (required for conversation features)
llm:
  primary:
    provider: "deepseek"
    model: "deepseek-chat"
    endpoint: "https://api.deepseek.com/v1"
    api_key: "${DEEPSEEK_API_KEY}"

# Web Terminal configuration
web:
  enabled: true
  bind: "127.0.0.1"  # Local access only by default
  port: 7788
  allowed_origins: ["*"]  # CORS configuration
```

**Important**: LLM configuration is required for AI conversation features. Otherwise, only system commands and Shell commands will work.

### Configuration Options

- **enabled**: Enable Web service (default: false)
- **bind**: Bind address
  - `127.0.0.1` - Local access only (secure)
  - `0.0.0.0` - Allow LAN access (use with caution)
  - Specific IP - Bind to specific network interface
- **port**: Port number (default: 7788)
- **allowed_origins**: CORS allowed origins

## Features

### Supported Operations

**System Commands** - All RealConsole commands starting with `/`
```
/help
/memory add "memory content"
/stats
```

**Shell Commands** - System commands starting with `!`
```
!ls -la
!pwd
!git status
```

**LLM Conversation** - Direct text input for AI interaction
```
Introduce Rust programming language
Translate this code
```

**Data Visualization** (v1.44.0+) - Chart commands starting with `!chart`
```
!chart line --title "Monthly Sales" --x-axis "Jan,Feb,Mar" --series "Sales:120,132,145"
!chart pie --title "Market Share" --labels "A,B,C" --series "Share:35,25,40"
```

> See [visualization-guide.md](visualization-guide.md) for complete visualization guide

### Keyboard Shortcuts

- **Ctrl+C** - Interrupt current operation
- **Ctrl+L** - Clear screen
- **Enter** - Execute command
- **Backspace** - Delete character

### Interface Features

- Modern UI with gradient backgrounds and rounded cards
- Responsive design supporting phones, tablets, and desktops
- Dark theme for eye comfort
- Real-time WebSocket communication with low latency

## Architecture

### Technology Stack

- **Backend**: axum + WebSocket
- **Frontend**: xterm.js + vanilla JavaScript
- **Protocol**: JSON message format

### Message Protocol

#### Client → Server

```json
{
  "type": "input",
  "content": "user command"
}
```

#### Server → Client

```json
// Normal output
{
  "type": "output",
  "content": "response text"
}

// Error message
{
  "type": "error",
  "content": "error message"
}

// Clear screen
{
  "type": "clear"
}
```

## Security Recommendations

### Important Notes

1. **Local access by default** - `bind: 127.0.0.1` only allows local access
2. **LAN access requires caution** - Using `0.0.0.0` exposes to LAN
3. **No authentication** - Current version doesn't support password protection
4. **Shell command execution** - Has system command execution permissions

### Best Practices

- Only use in trusted networks
- Use firewall to restrict access
- Regularly check connection logs
- Do not expose service to the internet
- Do not use in production environments

## Troubleshooting

### Cannot Access

**Local access fails**:
1. Check if service is started
2. Check if port is in use: `lsof -i :7788`
3. Try a different port: `realconsole web --port 8080`
4. Check browser console for errors

**LAN access fails**:
1. Confirm started with `--bind 0.0.0.0` or specific IP
2. Confirm both devices are on the same LAN (ping test)
3. Check firewall configuration
4. Try accessing `http://127.0.0.1:7788` on the server
5. Check if router isolates devices (AP isolation)

### Firewall Configuration

#### macOS

**Check firewall status**:
```bash
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate
```

**Configuration steps**:
1. System Preferences → Security & Privacy → Firewall
2. Click "Firewall Options"
3. Click + to add application
4. Select `realconsole` executable
5. Ensure set to "Allow incoming connections"

#### Linux (iptables)

```bash
# Allow port 7788
sudo iptables -A INPUT -p tcp --dport 7788 -j ACCEPT

# Only allow specific subnet (more secure)
sudo iptables -A INPUT -p tcp --dport 7788 -s 192.168.1.0/24 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 7788 -j DROP

# Save rules
sudo iptables-save > /etc/iptables/rules.v4
```

### WebSocket Connection Failed

1. Check browser console for errors (F12 → Console)
2. Check CORS configuration (default allows all origins)
3. Try different browsers (Chrome, Firefox, Safari)
4. Check proxy settings (may block WebSocket)

## Usage Examples

### Example 1: Local Development (Recommended)

```bash
# Start Web Terminal (default config)
realconsole web

# Open in browser
open http://localhost:7788
```

**Use case**: Personal development, testing, learning

### Example 2: LAN Collaboration

```bash
# Step 1: Get your IP
ifconfig | grep "inet " | grep -v 127.0.0.1
# Example output: 192.168.1.100

# Step 2: Start service (bind to all interfaces)
realconsole web --bind 0.0.0.0

# Step 3: Access from other devices
# Browser: http://192.168.1.100:7788
```

**Use cases**:
- Team demonstrations
- Mobile device access
- Cross-device debugging

### Example 3: Specific IP Binding (More Secure)

```bash
# Bind to specific IP only (recommended)
realconsole web --bind 192.168.1.100 --port 7788

# Access URLs
# Local: http://127.0.0.1:7788
# LAN: http://192.168.1.100:7788
```

**Advantage**: More secure than 0.0.0.0, only listens on specified interface

## Limitations and Future Plans

### Current Limitations

- No multi-user simultaneous login support
- No authentication mechanism
- No HTTPS support
- Streaming output not fully implemented

### Future Plans

- [ ] Add authentication (username/password)
- [ ] Support HTTPS/TLS
- [ ] Full streaming output support
- [ ] Session persistence
- [ ] Multi-tab support
- [ ] File upload/download functionality

## Related Resources

- [User Guide](./user-guide.md)
- [LLM Setup Guide](./llm-setup.en.md)
- [Quick Start](./quickstart.en.md)

---

**Version**: v1.52.0
**Updated**: 2026-01-08
