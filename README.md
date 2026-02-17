<div align="center">

# 🎮 VRChat Auto-Diary

**Transform your VRChat experiences into beautifully crafted diaries, novels, and artwork — all automatically.**

[![Rust](https://img.shields.io/badge/Rust-1.75+-000000?style=flat-square&logo=rust&logoColor=white)](https://rust-lang.org)
[![Next.js](https://img.shields.io/badge/Next.js-15-black?style=flat-square&logo=next.js)](https://nextjs.org)
[![Supabase](https://img.shields.io/badge/Supabase-PostgreSQL-3ecf8e?style=flat-square&logo=supabase)](https://supabase.com)
[![License](https://img.shields.io/badge/License-Private-gray?style=flat-square)]()

[Live Demo](https://kaflog.vercel.app) · [Documentation](docs/overview.md) · [Development Guide](AGENTS.md)

</div>

---

## ✨ Features

| Feature | Description |
|---------|-------------|
| 🎤 **Auto Recording** | Detects VRChat launch/exit and records audio automatically |
| 📝 **AI Transcription** | Gemini 3 Flash for high-accuracy speech-to-text |
| 📖 **Smart Summaries** | Gemini 3 Flash transforms conversations into diary entries |
| 📚 **Novel Generation** | Long-form narrative chapters from your daily experiences |
| 🎨 **AI Artwork** | Auto-generated illustrations matching your story's mood |
| ☁️ **Cloud Sync** | Seamless sync to Supabase with public/private control |
| 🌐 **Web Reader** | Modern Next.js frontend to browse your memories |

---

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+
- [Cargo](https://doc.rust-lang.org/cargo/) package manager
- [Task](https://taskfile.dev) runner

### Installation

```bash
# Clone and setup
git clone https://github.com/yourusername/vlog.git
cd vlog
cargo build

# Configure environment
cp .env.example .env
# Edit .env with your API keys
```

### Run

```bash
# Linux/WSL - Start as service
task up

# Windows - Double-click or run
windows\run.bat
```

---

## 📁 Project Structure

```
vlog/
├── src/                    # Rust backend
│   ├── infrastructure/     # API, audio, repositories
│   │   ├── audio.rs        # Recording management
│   │   ├── api.rs          # Gemini & Supabase clients
│   │   └── tasks.rs        # Task repository
│   ├── use_cases/          # Business logic (ProcessUseCase)
│   └── domain/             # Entities
├── frontend/reader/        # Next.js web app
├── data/                   # Local storage
│   ├── recordings/         # Audio files (FLAC)
│   ├── summaries/          # AI-generated diaries
│   ├── novels/             # Long-form chapters
│   └── photos/             # Generated artwork
└── docs/                   # Documentation
```

---

## 🔧 Commands

| Command | Description |
|---------|-------------|
| `task up` | Start systemd service |
| `task down` | Stop service |
| `task status` | Check system status |
| `task logs` | Real-time log streaming |
| `task process FILE=...` | Process single audio file |
| `task sync` | Sync to Supabase |
| `task web:dev` | Start frontend dev server |
| `task web:deploy` | Deploy to Vercel |

---

## 🛠️ Tech Stack

<table>
<tr>
<td align="center" width="96">
<b>Backend</b>
</td>
<td align="center" width="96">
<b>AI/ML</b>
</td>
<td align="center" width="96">
<b>Frontend</b>
</td>
<td align="center" width="96">
<b>Infra</b>
</td>
</tr>
<tr>
<td align="center">
Rust 1.75+<br/>
CPAL / Rodio<br/>
Serde
</td>
<td align="center">
Gemini 3 Flash<br/>
(Google AI)
</td>
<td align="center">
Next.js 15<br/>
React 19<br/>
TypeScript
</td>
<td align="center">
Supabase<br/>
Vercel<br/>
systemd
</td>
</tr>
</table>

---

## 📖 Documentation

| Document | Description |
|----------|-------------|
| [AGENTS.md](AGENTS.md) | Development guide & coding conventions |
| [docs/overview.md](docs/overview.md) | Complete system documentation |
| [docs/architecture.md](docs/architecture.md) | Visual system diagrams |
| [docs/image.md](docs/image.md) | Image generation subsystem |

---

## 🌐 Live

**Production**: [kaflog.vercel.app](https://kaflog.vercel.app)

---

<div align="center">

Made with ❤️ for VRChat memories

</div>
