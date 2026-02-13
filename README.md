# PopTranscribe

Application de transcription de reunions en temps reel, avec diarisation des speakers et resume automatique par IA. Disponible sur **macOS** et **Windows**.

## Fonctionnalites

- **Transcription en temps reel** — Audio capture et transcrit par Mistral Voxtral avec streaming SSE
- **Diarisation** — Identification automatique des speakers avec avatars uniques (facehash)
- **Resume IA** — Generation automatique d'un resume structure a la fin de chaque session
- **Chat IA** — Posez des questions sur la transcription en cours ou passee
- **Export** — Markdown, PDF
- **Recherche** — Recherche plein texte dans les transcriptions (FTS5)
- **Modes** — Visio (audio systeme) et Presentiel (microphone)

## Stack technique

| Couche | Technologie |
|--------|-------------|
| Desktop | [Tauri 2](https://v2.tauri.app/) |
| Backend | Rust (tokio, reqwest, rusqlite, cpal) |
| Frontend | React 19, TypeScript, Tailwind CSS 4 |
| Transcription | [Mistral Voxtral](https://docs.mistral.ai/) (streaming SSE) |
| Resume / Chat | Mistral Large |
| Base de donnees | SQLite (embarquee, FTS5) |
| Audio systeme | ScreenCaptureKit (macOS) / WASAPI loopback (Windows) |

## Telechargement

Telecharger le dernier installeur depuis la [page Releases](https://github.com/Kwickos/poptranscribe/releases) :

| Plateforme | Format |
|------------|--------|
| macOS | `.dmg` |
| Windows | `.msi` / `.exe` |

## Pre-requis (build depuis les sources)

- macOS 13+ ou Windows 10+
- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 18+
- Une cle API [Mistral](https://console.mistral.ai/)

## Installation depuis les sources

```bash
git clone https://github.com/Kwickos/poptranscribe.git
cd poptranscribe
npm install
npm run tauri dev
```

Pour builder l'installeur :

```bash
npm run tauri build
```

Les artefacts seront dans `src-tauri/target/release/bundle/`.

## Configuration

Au premier lancement, aller dans **Reglages** et entrer votre cle API Mistral.

## Branches

| Branche | Role |
|---------|------|
| `main` | Releases stables |
| `beta` | Pre-releases / tests |
| `develop` | Developpement quotidien |

## Architecture

```
poptranscribe/
├── src/                    # Frontend React
│   ├── components/         # Composants UI (ChatPanel, TranscriptLine, etc.)
│   ├── views/              # Vues principales (SessionView, DetailView, etc.)
│   └── styles.css          # Styles globaux + animations
├── src-tauri/              # Backend Rust
│   └── src/
│       ├── audio/          # Capture audio (systeme + micro)
│       ├── mistral/        # Client API Mistral (streaming + batch)
│       ├── db/             # SQLite + FTS5
│       └── commands.rs     # Commandes Tauri
├── .github/workflows/      # CI/CD (build macOS + Windows)
└── package.json
```

## Licence

Projet prive.
