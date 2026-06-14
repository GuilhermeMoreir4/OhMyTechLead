# oh-my-tech-lead

A terminal app to track your daily work tasks and automatically send a report to your tech lead via Discord or WhatsApp — before the standup.

```
┌─ oh-my-tech-lead ────────────────── Fri, 13 Jun 2026 ─┐
│                                                        │
│  ┌── 📋 Review (2) ──────┐  ┌── ✅ Done (3) ─────────┐ │
│  │ > Review João's PR    │  │   Implemented screen X  │ │
│  │   Maria's PR          │  │   Fixed bug #42         │ │
│  └───────────────────────┘  └────────────────────────┘ │
│  ┌── 🔄 Back from Review ┐  ┌── 🚧 Blocker (0) ──────┐ │
│  │   Task Y              │  │   (empty)               │ │
│  └───────────────────────┘  └────────────────────────┘ │
│                                                        │
│  [N] New  [D] Delete  [P] Preview  [S] Send  [Q] Quit │
└────────────────────────────────────────────────────────┘
```

---

## Features

- **4 task categories:** Review, Done, Back from Review, Blocker
- **Interactive TUI** with keyboard navigation (ratatui)
- **Report preview** before sending
- **Send via Discord DM** directly to the tech lead
- **Send via WhatsApp** using a self-hosted WAHA container (auto-managed via Docker)
- **Automatic scheduling** Monday–Friday via systemd daemon
- **Manual send** with a single command
- **In-TUI settings** — configure everything without leaving the app
- **Toggle Discord / WhatsApp** independently

---

## Installation

```bash
git clone git@github.com:GuilhermeMoreir4/OhMyTechLead.git
cd OhMyTechLead
./install.sh
```

The script handles everything automatically:
- Installs Rust/cargo via rustup if not present
- Compiles in release mode and installs the `omtl` binary to `~/.cargo/bin`
- Optionally runs `omtl setup` to configure the bot and schedule

### Manual installation

```bash
cargo install --path .   # builds and installs to ~/.cargo/bin/omtl
omtl setup               # configure and activate the daemon
```

### Build from source

```bash
cargo build --release
cp target/release/omtl ~/.local/bin/
```

---

## Quick Start

```bash
omtl          # open the TUI
omtl send     # send today's report immediately
omtl daemon   # run the scheduling daemon manually
omtl setup    # re-run the configuration wizard
```

---

## Discord Setup

### 1. Create the bot

1. Go to [discord.com/developers/applications](https://discord.com/developers/applications)
2. Click **New Application** → give it a name (e.g. `omtl-bot`)
3. In the sidebar, click **Bot**
4. Click **Reset Token** and copy the generated token

### 2. Invite the bot to a shared server

The bot must share **at least one server** with the tech lead to be able to send DMs.

1. Go to **OAuth2 → URL Generator**
2. Select the **bot** scope
3. Check **Send Messages** permission
4. Open the generated URL in a browser and invite the bot to a shared server

### 3. Get the tech lead's User ID

1. In Discord, go to **Settings → Advanced**
2. Enable **Developer Mode**
3. Right-click the tech lead's profile → **Copy User ID**

### 4. Configure in the TUI

Press `[C]` inside `omtl` to open Settings, then fill in:
- **Discord — Bot Token**
- **Discord — Tech Lead User ID**

Toggle **Discord — Enabled** on, then press `Ctrl+S` to save.

---

## WhatsApp Setup

WhatsApp delivery uses [WAHA](https://waha.devlike.pro/) (free Core tier), auto-managed as a local Docker container.

**Requirements:** Docker must be installed and running.

### Setup

1. Press `[W]` inside `omtl`
2. The app starts the WAHA container automatically
3. A QR code appears in the terminal and **opens automatically in your browser** as a pixel-perfect image
4. Scan the QR with WhatsApp → **Linked Devices → Link a Device**
5. Enter the tech lead's phone number with country code (e.g. `5511999999999`)

The session is persisted in a Docker volume — you only need to scan once.

---

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `N` | New task |
| `D` | Delete selected task |
| `P` | Preview report |
| `S` | Send report now |
| `C` | Open settings |
| `W` | WhatsApp setup |
| `Tab` / `→` | Next category |
| `Shift+Tab` / `←` | Previous category |
| `↑` / `↓` | Navigate tasks |
| `Q` | Quit |

**Inside Settings:**

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate fields |
| `Enter` | Edit field / toggle |
| `Space` | Toggle on/off |
| `Ctrl+S` | Save |
| `Esc` | Back |

---

## Report Format

The report sent via Discord uses Markdown formatting:

```
📊 **Daily report — Fri, 13 Jun 2026**

📋 **Review:**
• Review João's PR
• Maria's PR

✅ **Done:**
• Implemented registration screen
• Fixed bug #42
• Deploy to staging

🔄 **Back from Review:**
• Task Y

🚧 **Blocker:**
(none)
```

---

## Configuration

All settings are stored in `~/.config/omtl/config.toml`:

```toml
[discord]
enabled = true
bot_token = "Bot MTxxxx..."
tech_lead_user_id = "123456789"

[whatsapp]
enabled = false
evolution_url = "http://127.0.0.1:3000"
api_key = "omtl-local-key"
instance = "default"
tech_lead_phone = "5511999999999"

[schedule]
send_time = "09:00"
```

| Path | Contents |
|------|----------|
| `~/.config/omtl/config.toml` | Settings (tokens, IDs, schedule) |
| `~/.local/share/omtl/tasks/YYYY-MM-DD.json` | Daily tasks |
| `~/.local/share/omtl/sent.log` | Last send date (prevents duplicate sends) |

---

## systemd Daemon

`omtl setup` installs and enables a user systemd service automatically. To manage it:

```bash
systemctl --user status omtl      # check status
systemctl --user restart omtl     # restart after config changes
journalctl --user -u omtl -f      # follow logs
systemctl --user disable --now omtl  # disable completely
```

---

## Project Structure

```
src/
├── main.rs        — Entrypoint and CLI (clap)
├── app.rs         — Application state and TUI state machine
├── tui.rs         — Event loop and keyboard handlers
├── storage.rs     — Task persistence (JSON) and sent.log
├── config.rs      — config.toml read/write
├── report.rs      — Report text generation
├── discord.rs     — Discord DM via REST API
├── whatsapp.rs    — WhatsApp via WAHA REST API + QR rendering
├── wpp_setup.rs   — WhatsApp setup flow (Docker + QR + session)
├── docker.rs      — Docker container management for WAHA
├── scheduler.rs   — Scheduling daemon and manual send
└── ui/
    ├── mod.rs           — Screen routing
    ├── dashboard.rs     — 2×2 category grid
    ├── add_task.rs      — New task modal
    ├── preview.rs       — Report preview
    ├── settings.rs      — Settings screen
    └── whatsapp_setup.rs — WhatsApp setup screen
```

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ratatui` | TUI framework |
| `crossterm` | Cross-platform terminal backend |
| `tokio` | Async runtime |
| `reqwest` | HTTP client (Discord & WAHA APIs) |
| `chrono` | Date/time and weekday checks |
| `clap` | CLI argument parsing |
| `serde` / `serde_json` | Task serialization |
| `toml` | Config parsing |
| `directories` | XDG paths (`~/.config`, `~/.local/share`) |
| `qrcode` | QR code rendering in terminal |
| `uuid` | Unique task IDs |
| `anyhow` | Error handling |

---

## License

MIT
