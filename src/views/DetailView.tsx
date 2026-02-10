import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { SessionDetail } from '../types';
import DiarizedTranscript from '../components/DiarizedTranscript';
import SummaryPanel from '../components/SummaryPanel';
import ExportButtons from '../components/ExportButtons';

function formatDate(iso: string): string {
  const date = new Date(iso);
  return date.toLocaleDateString('fr-FR', {
    day: 'numeric',
    month: 'long',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function formatDuration(secs: number | null): string {
  if (secs === null || secs === 0) return '--';
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = Math.floor(secs % 60);
  if (h > 0) {
    return `${h}h ${String(m).padStart(2, '0')}min`;
  }
  return `${m}min ${String(s).padStart(2, '0')}s`;
}

type DetailTab = 'transcript' | 'summary';

interface DetailViewProps {
  sessionId: string;
  onBack: () => void;
}

export default function DetailView({ sessionId, onBack }: DetailViewProps) {
  const [detail, setDetail] = useState<SessionDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [editingTitle, setEditingTitle] = useState(false);
  const [titleDraft, setTitleDraft] = useState('');

  const [activeTab, setActiveTab] = useState<DetailTab>('transcript');

  useEffect(() => {
    if (sessionId) {
      setLoading(true);
      setError(null);
      invoke<SessionDetail>('get_session_detail', { sessionId })
        .then((data) => {
          setDetail(data);
          setTitleDraft(data.title);
        })
        .catch((err) => setError(String(err)))
        .finally(() => setLoading(false));
    }
  }, [sessionId]);

  const handleTitleSave = useCallback(async () => {
    if (!detail || !sessionId) return;
    const trimmed = titleDraft.trim();
    if (!trimmed || trimmed === detail.title) {
      setEditingTitle(false);
      setTitleDraft(detail.title);
      return;
    }
    try {
      await invoke('update_session_title', { sessionId, title: trimmed });
      setDetail((prev) => (prev ? { ...prev, title: trimmed } : prev));
    } catch {
      setTitleDraft(detail.title);
    }
    setEditingTitle(false);
  }, [detail, sessionId, titleDraft]);

  const handleSpeakerRenamed = useCallback(
    (oldName: string, newName: string) => {
      setDetail((prev) => {
        if (!prev) return prev;
        return {
          ...prev,
          segments: prev.segments.map((seg) =>
            seg.speaker === oldName ? { ...seg, speaker: newName } : seg
          ),
        };
      });
    },
    []
  );

  // Loading state
  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <svg className="w-7 h-7 animate-spin text-gray-300" fill="none" viewBox="0 0 24 24">
          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
        </svg>
        <span className="text-sm text-gray-400">Chargement...</span>
      </div>
    );
  }

  // Error state
  if (error || !detail) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-5">
        <div className="w-14 h-14 rounded-2xl bg-red-50 flex items-center justify-center">
          <svg className="w-7 h-7 text-red-300" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v3.75m9-.75a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9 3.75h.008v.008H12v-.008Z" />
          </svg>
        </div>
        <p className="text-sm text-gray-500">
          {error ?? 'Session introuvable'}
        </p>
        <button
          onClick={onBack}
          className="flex items-center gap-2 px-5 py-2.5 bg-gray-900 hover:bg-gray-800 text-white text-sm font-medium rounded-xl transition-all duration-150"
        >
          Retour
        </button>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full p-6">
      {/* Header */}
      <header className="shrink-0 mb-5">
        {/* Title row */}
        <div className="flex items-start justify-between gap-4 mb-3">
          <div className="flex-1 min-w-0">
            {editingTitle ? (
              <input
                type="text"
                value={titleDraft}
                onChange={(e) => setTitleDraft(e.target.value)}
                onBlur={handleTitleSave}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleTitleSave();
                  if (e.key === 'Escape') {
                    setTitleDraft(detail.title);
                    setEditingTitle(false);
                  }
                }}
                autoFocus
                className="text-base font-medium text-gray-900 bg-transparent border-b border-gray-200 outline-none w-full pb-0.5"
              />
            ) : (
              <h1
                onClick={() => setEditingTitle(true)}
                className="text-base font-medium text-gray-900 cursor-pointer hover:text-gray-600 transition-colors truncate"
                title="Cliquer pour modifier le titre"
              >
                {detail.title}
              </h1>
            )}
          </div>
          <ExportButtons sessionId={detail.id} />
        </div>

        {/* Metadata + tab toggle — inline floating pills */}
        <div className="flex flex-wrap items-center gap-2 mt-2">
          <span className="text-xs text-gray-400">{formatDate(detail.created_at)}</span>
          <span className="text-gray-200">·</span>
          <span className="text-xs text-gray-400">{formatDuration(detail.duration_secs)}</span>
          <span className="text-gray-200">·</span>
          <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-medium bg-white/80 text-gray-400 shadow-sm capitalize">
            {detail.mode}
          </span>

          <div className="ml-auto flex bg-white/80 rounded-full p-0.5 shadow-sm">
            <button
              onClick={() => setActiveTab('transcript')}
              className={`px-3.5 py-1 text-xs font-medium rounded-full transition-all duration-150 ${
                activeTab === 'transcript'
                  ? 'bg-gray-900 text-white'
                  : 'text-gray-400 hover:text-gray-600'
              }`}
            >
              Transcription
            </button>
            <button
              onClick={() => setActiveTab('summary')}
              className={`px-3.5 py-1 text-xs font-medium rounded-full transition-all duration-150 ${
                activeTab === 'summary'
                  ? 'bg-gray-900 text-white'
                  : 'text-gray-400 hover:text-gray-600'
              }`}
            >
              Resume
            </button>
          </div>
        </div>
      </header>

      {/* Content area */}
      <div className="flex-1 min-h-0 overflow-auto pr-2">
        {activeTab === 'transcript' ? (
          <div className="pb-6">
            <DiarizedTranscript
              sessionId={detail.id}
              segments={detail.segments}
              onSpeakerRenamed={handleSpeakerRenamed}
            />
          </div>
        ) : (
          <SummaryPanel summary={detail.summary} />
        )}
      </div>
    </div>
  );
}
