import { useState, useEffect, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
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

type ActiveTab = 'transcript' | 'summary';

export default function DetailView() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const [detail, setDetail] = useState<SessionDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [editingTitle, setEditingTitle] = useState(false);
  const [titleDraft, setTitleDraft] = useState('');

  const [activeTab, setActiveTab] = useState<ActiveTab>('transcript');

  useEffect(() => {
    if (id) {
      setLoading(true);
      setError(null);
      invoke<SessionDetail>('get_session_detail', { sessionId: id })
        .then((data) => {
          setDetail(data);
          setTitleDraft(data.title);
        })
        .catch((err) => setError(String(err)))
        .finally(() => setLoading(false));
    }
  }, [id]);

  const handleTitleSave = useCallback(async () => {
    if (!detail || !id) return;
    const trimmed = titleDraft.trim();
    if (!trimmed || trimmed === detail.title) {
      setEditingTitle(false);
      setTitleDraft(detail.title);
      return;
    }
    try {
      await invoke('update_session_title', { sessionId: id, title: trimmed });
      setDetail((prev) => (prev ? { ...prev, title: trimmed } : prev));
    } catch {
      setTitleDraft(detail.title);
    }
    setEditingTitle(false);
  }, [detail, id, titleDraft]);

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
      <div className="flex items-center justify-center h-full">
        <div className="flex items-center gap-3 text-gray-400">
          <svg
            className="animate-spin h-5 w-5"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
          >
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
              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            />
          </svg>
          <span className="text-sm">Chargement...</span>
        </div>
      </div>
    );
  }

  // Error state
  if (error || !detail) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <p className="text-red-600 text-sm">
          {error ?? 'Session introuvable'}
        </p>
        <button
          onClick={() => navigate('/history')}
          className="px-4 py-2 bg-gray-100 hover:bg-gray-200 text-gray-700 text-sm rounded-lg transition-colors"
        >
          Retour a l'historique
        </button>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full max-h-full">
      {/* Header */}
      <header className="shrink-0 pb-4 border-b border-gray-200 mb-4">
        <div className="flex items-start gap-3">
          <button
            onClick={() => navigate('/history')}
            className="mt-1 p-1.5 rounded-lg text-gray-400 hover:text-gray-900 hover:bg-gray-100 transition-colors"
            title="Retour a l'historique"
          >
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M15 19l-7-7 7-7" />
            </svg>
          </button>

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
                className="text-xl font-semibold text-gray-900 bg-transparent border-b-2 border-gray-400 outline-none w-full pb-0.5"
              />
            ) : (
              <h2
                onClick={() => setEditingTitle(true)}
                className="text-xl font-semibold text-gray-900 cursor-pointer hover:text-gray-600 transition-colors truncate"
                title="Cliquer pour modifier le titre"
              >
                {detail.title}
              </h2>
            )}

            <div className="flex items-center gap-3 mt-1 text-xs text-gray-400">
              <span>{formatDate(detail.created_at)}</span>
              <span className="w-1 h-1 rounded-full bg-gray-300" />
              <span>{formatDuration(detail.duration_secs)}</span>
              <span className="w-1 h-1 rounded-full bg-gray-300" />
              <span className="capitalize">{detail.mode}</span>
            </div>
          </div>
        </div>

        {/* Tabs for mobile / narrow layout */}
        <div className="flex gap-1 mt-4 bg-gray-100 rounded-lg p-0.5 lg:hidden">
          <button
            onClick={() => setActiveTab('transcript')}
            className={`px-3 py-1.5 text-xs font-medium rounded-md transition-colors ${
              activeTab === 'transcript'
                ? 'bg-white text-gray-900 shadow-sm'
                : 'text-gray-500 hover:text-gray-700'
            }`}
          >
            Transcription
          </button>
          <button
            onClick={() => setActiveTab('summary')}
            className={`px-3 py-1.5 text-xs font-medium rounded-md transition-colors ${
              activeTab === 'summary'
                ? 'bg-white text-gray-900 shadow-sm'
                : 'text-gray-500 hover:text-gray-700'
            }`}
          >
            Resume
          </button>
        </div>
      </header>

      {/* Main content: two columns on large screens, tabs on small */}
      <div className="flex-1 min-h-0 flex gap-4">
        {/* Transcript column */}
        <div
          className={`flex-1 min-w-0 bg-white border border-gray-200 rounded-xl overflow-auto ${
            activeTab !== 'transcript' ? 'hidden lg:block' : ''
          }`}
        >
          <div className="sticky top-0 bg-white/95 backdrop-blur-sm border-b border-gray-200 px-4 py-2.5">
            <h3 className="text-sm font-medium text-gray-900">Transcription</h3>
            <p className="text-xs text-gray-400 mt-0.5">
              {detail.segments.length} segment{detail.segments.length !== 1 ? 's' : ''}
            </p>
          </div>
          <div className="p-2">
            <DiarizedTranscript
              sessionId={detail.id}
              segments={detail.segments}
              onSpeakerRenamed={handleSpeakerRenamed}
            />
          </div>
        </div>

        {/* Summary sidebar */}
        <div
          className={`lg:w-80 lg:shrink-0 bg-white border border-gray-200 rounded-xl overflow-auto ${
            activeTab !== 'summary' ? 'hidden lg:block' : 'flex-1'
          }`}
        >
          <div className="sticky top-0 bg-white/95 backdrop-blur-sm border-b border-gray-200 px-4 py-2.5">
            <h3 className="text-sm font-medium text-gray-900">Resume</h3>
          </div>
          <SummaryPanel summary={detail.summary} />
        </div>
      </div>

      {/* Bottom bar: export buttons */}
      <footer className="shrink-0 pt-4 mt-4 border-t border-gray-200">
        <ExportButtons sessionId={detail.id} />
      </footer>
    </div>
  );
}
