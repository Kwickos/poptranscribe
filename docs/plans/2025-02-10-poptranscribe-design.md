# PopTranscribe - Design Document

## Overview

App macOS de transcription de reunions utilisant Mistral AI (Voxtral Mini).
Deux phases par reunion : transcription temps reel avec recherche pendant la reunion,
puis retraitement batch avec diarisation apres la reunion.

## Stack

- **Tauri** (Rust backend + React/TypeScript frontend)
- **Mistral AI** : Voxtral Mini pour la transcription, modele chat pour recherche/resume
- **SQLite** pour le stockage local
- **macOS APIs** : ScreenCaptureKit + AVAudioEngine

## APIs Mistral

| API | Modele | Usage | Cout |
|---|---|---|---|
| Transcription SSE streaming | `voxtral-mini-latest` | Transcription temps reel | $0.006/min |
| Transcription batch + diarisation | `voxtral-mini-latest` | Retraitement post-reunion | $0.003/min |
| Chat completions | Mistral Small/Nemo | Recherche langage naturel + resume | variable |

## Capture audio

Deux modes selectionnables dans l'app :

| Mode | Sources | Techno |
|---|---|---|
| Visio | Audio systeme + Micro | ScreenCaptureKit + AVAudioEngine, mixage en Rust |
| Presentiel | Micro uniquement | AVAudioEngine |

- Format : PCM 16-bit mono, 16kHz
- Deux flux paralleles : streaming vers Mistral + sauvegarde locale (WAV)
- Permissions macOS : Screen Recording (visio) + Microphone (presentiel)

## Interface utilisateur

L'app vit dans la menu bar macOS. Trois vues :

### 1. Vue Session (pendant la reunion)

- Bouton Start/Stop
- Toggle Visio / Presentiel
- Transcription live qui defile (un segment par ligne avec timestamp)
- Barre de recherche :
  - Texte classique avec surlignage
  - Prefixe `?` ou bouton dedie pour recherche LLM
  - Reponses LLM dans un panneau lateral
- Indicateur temps ecoule + niveau audio

### 2. Vue Historique

- Liste des reunions (date, duree, titre auto-genere ou editable)
- Recherche globale dans toutes les reunions

### 3. Vue Detail (post-reunion)

- Transcription diarisee avec timestamps ("Speaker 1: ...")
- Renommage des speakers (Speaker 1 -> Alexandre)
- Resume : points cles, decisions, actions a suivre
- Export : Markdown, PDF, Notion, Slack

## Pipeline post-reunion

Sequence automatique au clic sur Stop :

1. Arret capture, finalisation fichier audio
2. Envoi batch a Voxtral Mini avec `diarize: true`
3. Generation resume via Mistral chat (points cles, decisions, actions)
4. Stockage en SQLite + notification macOS
5. Export disponible immediatement

## Architecture technique

```
+---------------------------------------------------+
|                   Tauri App                        |
|                                                    |
|  +---------------+       +----------------------+  |
|  |  React/TS UI  |<----->|   Tauri Commands     |  |
|  |               |       |   (Rust bridge)      |  |
|  +---------------+       +-----------+----------+  |
|                                      |             |
|  +-----------------------------------+----------+  |
|  |              Rust Backend                    |  |
|  |                                              |  |
|  |  +--------------+  +-----------------------+ |  |
|  |  | Audio Core   |  |   Mistral Client      | |  |
|  |  |              |  |                       | |  |
|  |  |ScreenCapKit  |  | - SSE streaming       | |  |
|  |  |+ AVAudio     |  | - Batch + diarize     | |  |
|  |  |+ Mixer       |  | - Chat (recherche)    | |  |
|  |  +------+-------+  +-----------+-----------+ |  |
|  |         |                      |             |  |
|  |         v                      v             |  |
|  |  +--------------+  +-----------------------+ |  |
|  |  | File Store   |  |     SQLite DB         | |  |
|  |  | (WAV)        |  | (sessions, textes,    | |  |
|  |  |              |  |  resumes, settings)   | |  |
|  |  +--------------+  +-----------------------+ |  |
|  +----------------------------------------------+  |
+---------------------------------------------------+
```

### Modules Rust

| Module | Role |
|---|---|
| `audio_capture` | ScreenCaptureKit + AVAudioEngine, mixage, encoding PCM |
| `audio_store` | Sauvegarde fichier audio local |
| `mistral_realtime` | Client SSE pour transcription live |
| `mistral_batch` | Client REST pour transcription diarisee |
| `mistral_chat` | Client REST pour recherche LN + resume |
| `db` | SQLite via rusqlite, CRUD sessions/transcriptions |
| `export` | Generation Markdown, PDF, envoi Notion/Slack |

### Composants React/TS

| Composant | Role |
|---|---|
| `SessionView` | Transcription live + recherche |
| `HistoryView` | Liste des reunions |
| `DetailView` | Transcription diarisee + resume + export |
| `MenuBar` | Integration menu bar macOS |

## Cout estime par reunion

- Reunion 1h temps reel : ~$0.36
- Retraitement batch 1h : ~$0.18
- Resume/recherche LLM : ~$0.01-0.05
- **Total reunion 1h : ~$0.55-0.60**
- **20 reunions/mois : ~$11-12/mois**
