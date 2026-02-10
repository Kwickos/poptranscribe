import { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { Segment } from '../types';
import TranscriptLine from '../components/TranscriptLine';
import SearchBar from '../components/SearchBar';
import AudioLevel from '../components/AudioLevel';

function formatElapsedTime(totalSeconds: number): string {
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
}

export default function SessionView() {
  const [mode, setMode] = useState<'visio' | 'presentiel'>('visio');
  const [isRecording, setIsRecording] = useState(false);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [segments, setSegments] = useState<Segment[]>([]);
  const [elapsedTime, setElapsedTime] = useState(0);
  const [searchQuery, setSearchQuery] = useState('');
  const [llmResult, setLlmResult] = useState<string | null>(null);
  const [llmSearching, setLlmSearching] = useState(false);
  const [audioLevel, setAudioLevel] = useState(0);
  const [showLlmPanel, setShowLlmPanel] = useState(false);

  const transcriptEndRef = useRef<HTMLDivElement>(null);
  const transcriptContainerRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new segments arrive
  useEffect(() => {
    if (transcriptEndRef.current && !searchQuery) {
      transcriptEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [segments, searchQuery]);

  // Listen for live transcription events
  useEffect(() => {
    const unlistenSegment = listen<Segment>('transcription-segment', (event) => {
      setSegments((prev) => [...prev, event.payload]);
    });

    const unlistenAudio = listen<{ level: number }>('audio-level', (event) => {
      setAudioLevel(event.payload.level);
    });

    return () => {
      unlistenSegment.then((fn) => fn());
      unlistenAudio.then((fn) => fn());
    };
  }, []);

  // Timer
  useEffect(() => {
    if (!isRecording) return;
    const interval = setInterval(() => {
      setElapsedTime((prev) => prev + 1);
    }, 1000);
    return () => clearInterval(interval);
  }, [isRecording]);

  const handleStart = useCallback(async () => {
    try {
      const id = await invoke<string>('start_session', { mode });
      setSessionId(id);
      setIsRecording(true);
      setSegments([]);
      setElapsedTime(0);
      setSearchQuery('');
      setLlmResult(null);
      setShowLlmPanel(false);
    } catch (err) {
      console.error('Erreur au demarrage de la session:', err);
    }
  }, [mode]);

  const handleStop = useCallback(async () => {
    if (sessionId) {
      try {
        await invoke('stop_session', { sessionId });
      } catch (err) {
        console.error('Erreur a l\'arret de la session:', err);
      }
      setIsRecording(false);
      setSessionId(null);
      setAudioLevel(0);
    }
  }, [sessionId]);

  const handleTextSearch = useCallback((query: string) => {
    setSearchQuery(query);
  }, []);

  const handleLlmSearch = useCallback(
    async (query: string) => {
      if (!sessionId) return;
      setLlmSearching(true);
      setShowLlmPanel(true);
      try {
        const result = await invoke<string>('search_llm', { query, sessionId });
        setLlmResult(result);
      } catch (err) {
        console.error('Erreur lors de la recherche IA:', err);
        setLlmResult('Une erreur est survenue lors de la recherche.');
      } finally {
        setLlmSearching(false);
      }
    },
    [sessionId]
  );

  // Filter segments based on text search
  const filteredSegments = searchQuery
    ? segments.filter((s) => s.text.toLowerCase().includes(searchQuery.toLowerCase()))
    : segments;

  return (
    <div className="flex h-full gap-0">
      {/* Main content */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* Header */}
        <div className="flex flex-wrap items-center gap-4 pb-4 border-b border-gray-800">
          {/* Mode toggle */}
          <div className="flex bg-gray-800 rounded-lg p-0.5">
            <button
              onClick={() => setMode('visio')}
              disabled={isRecording}
              className={`px-4 py-1.5 rounded-md text-sm font-medium transition-colors ${
                mode === 'visio'
                  ? 'bg-indigo-600 text-white shadow-sm'
                  : 'text-gray-400 hover:text-white'
              } ${isRecording ? 'cursor-not-allowed' : ''}`}
            >
              Visio
            </button>
            <button
              onClick={() => setMode('presentiel')}
              disabled={isRecording}
              className={`px-4 py-1.5 rounded-md text-sm font-medium transition-colors ${
                mode === 'presentiel'
                  ? 'bg-indigo-600 text-white shadow-sm'
                  : 'text-gray-400 hover:text-white'
              } ${isRecording ? 'cursor-not-allowed' : ''}`}
            >
              Presentiel
            </button>
          </div>

          {/* Start/Stop button */}
          {!isRecording ? (
            <button
              onClick={handleStart}
              className="flex items-center gap-2 px-5 py-2 bg-green-600 hover:bg-green-500 text-white rounded-lg font-medium transition-colors shadow-sm"
            >
              <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                <circle cx="12" cy="12" r="8" />
              </svg>
              Demarrer
            </button>
          ) : (
            <button
              onClick={handleStop}
              className="flex items-center gap-2 px-5 py-2 bg-red-600 hover:bg-red-500 text-white rounded-lg font-medium transition-colors shadow-sm"
            >
              <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                <rect x="6" y="6" width="12" height="12" rx="2" />
              </svg>
              Arreter
            </button>
          )}

          {/* Timer */}
          <div className="font-mono text-lg text-gray-300 tabular-nums">
            {formatElapsedTime(elapsedTime)}
          </div>

          {/* Audio level */}
          <AudioLevel level={audioLevel} isActive={isRecording} />
        </div>

        {/* Search bar */}
        <div className="py-3">
          <SearchBar
            onTextSearch={handleTextSearch}
            onLlmSearch={handleLlmSearch}
            isSearching={llmSearching}
          />
        </div>

        {/* Transcript area */}
        <div
          ref={transcriptContainerRef}
          className="flex-1 overflow-auto border border-gray-800 rounded-lg p-3 min-h-0"
        >
          {filteredSegments.length === 0 ? (
            <div className="flex items-center justify-center h-full">
              <p className="text-gray-500 text-sm">
                {isRecording
                  ? 'En attente de la transcription...'
                  : segments.length === 0
                    ? 'La transcription apparaitra ici une fois la session demarree.'
                    : 'Aucun resultat pour cette recherche.'}
              </p>
            </div>
          ) : (
            <div className="space-y-0.5">
              {filteredSegments.map((segment) => (
                <TranscriptLine
                  key={segment.id}
                  text={segment.text}
                  startTime={segment.start_time}
                  highlight={searchQuery || undefined}
                />
              ))}
              <div ref={transcriptEndRef} />
            </div>
          )}
        </div>
      </div>

      {/* LLM results side panel */}
      {showLlmPanel && (
        <div className="w-80 border-l border-gray-800 ml-4 pl-4 flex flex-col min-h-0">
          <div className="flex items-center justify-between pb-3 border-b border-gray-800">
            <h3 className="text-sm font-semibold text-gray-300">Resultat IA</h3>
            <button
              onClick={() => setShowLlmPanel(false)}
              className="text-gray-500 hover:text-gray-300 transition-colors"
              title="Fermer"
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>
          <div className="flex-1 overflow-auto pt-3">
            {llmSearching ? (
              <div className="flex items-center gap-2 text-gray-400 text-sm">
                <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                  <circle
                    className="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    strokeWidth="4"
                  />
                  <path
                    className="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
                  />
                </svg>
                Recherche en cours...
              </div>
            ) : llmResult ? (
              <p className="text-sm text-gray-300 leading-relaxed whitespace-pre-wrap">
                {llmResult}
              </p>
            ) : null}
          </div>
        </div>
      )}
    </div>
  );
}
