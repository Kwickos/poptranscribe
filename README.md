# PopTranscribe

Application macOS de transcription de reunions en temps reel, avec diarisation des speakers et resume automatique par IA.

## Fonctionnalites

- **Transcription en temps reel** — Audio systeme capture via ScreenCaptureKit, transcrit par Mistral Voxtral avec streaming SSE
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
| Backend | Rust (tokio, reqwest, rusqlite, cpal, screencapturekit) |
| Frontend | React 19, TypeScript, Tailwind CSS 4 |
| Transcription | [Mistral Voxtral](https://docs.mistral.ai/) (streaming SSE) |
| Resume / Chat | Mistral Large |
| Base de donnees | SQLite (embarquee, FTS5) |

## Pre-requis

- macOS 13+ (Ventura)
- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 18+
- Une cle API [Mistral](https://console.mistral.ai/)

## Installation

### Depuis le DMG

Telecharger le `.dmg` depuis la [page Releases](https://github.com/Kwickos/poptranscribe/releases), ouvrir et glisser l'app dans Applications.

### Depuis les sources

```bash
git clone https://github.com/Kwickos/poptranscribe.git
cd poptranscribe
npm install
npm run tauri dev
```

Pour builder le `.dmg` :

```bash
npm run tauri build
```

Le DMG sera dans `src-tauri/target/release/bundle/dmg/`.

## Configuration

Au premier lancement, aller dans **Reglages** et entrer votre cle API Mistral.

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
└── package.json
```

## Licence

Projet prive.
