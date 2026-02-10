import { useState, useMemo } from 'react';
import { Facehash } from 'facehash';
import type { Segment } from '../types';
import SpeakerEditor from './SpeakerEditor';

const AVATAR_COLORS = [
  '#6366f1', '#8b5cf6', '#ec4899', '#f43f5e',
  '#f97316', '#eab308', '#22c55e', '#14b8a6',
  '#06b6d4', '#3b82f6', '#a855f7', '#e11d48',
];

function formatTimestamp(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;
}

interface DiarizedTranscriptProps {
  sessionId: string;
  segments: Segment[];
  onSpeakerRenamed: (oldName: string, newName: string) => void;
}

export default function DiarizedTranscript({
  sessionId,
  segments,
  onSpeakerRenamed,
}: DiarizedTranscriptProps) {
  const [editingSpeaker, setEditingSpeaker] = useState<{
    name: string;
    segmentId: number;
  } | null>(null);

  const speakers = useMemo(() => {
    const unique: string[] = [];
    for (const seg of segments) {
      const name = seg.speaker ?? 'Inconnu';
      if (!unique.includes(name)) {
        unique.push(name);
      }
    }
    return unique;
  }, [segments]);

  // First speaker is assumed to be the user
  const userSpeaker = speakers.length > 0 ? speakers[0] : null;

  if (segments.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full py-12">
        <div className="w-12 h-12 rounded-2xl bg-gray-50 flex items-center justify-center mb-3">
          <svg className="w-6 h-6 text-gray-300" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" d="M7.5 8.25h9m-9 3H12m-9.75 1.51c0 1.6 1.123 2.994 2.707 3.227 1.129.166 2.27.293 3.423.379.35.026.67.21.865.501L12 21l2.755-4.133a1.14 1.14 0 0 1 .865-.501 48.172 48.172 0 0 0 3.423-.379c1.584-.233 2.707-1.626 2.707-3.228V6.741c0-1.602-1.123-2.995-2.707-3.228A48.394 48.394 0 0 0 12 3c-2.392 0-4.744.175-7.043.513C3.373 3.746 2.25 5.14 2.25 6.741v6.018Z" />
          </svg>
        </div>
        <p className="text-sm text-gray-400">Aucun segment de transcription disponible.</p>
      </div>
    );
  }

  return (
    <div className="space-y-4 overflow-auto">
      {segments.map((seg) => {
        const speakerName = seg.speaker ?? 'Inconnu';
        const isUser = speakerName === userSpeaker;
        const isEditing =
          editingSpeaker !== null &&
          editingSpeaker.segmentId === seg.id;

        return (
          <div
            key={seg.id}
            className={`flex gap-2.5 ${isUser ? 'flex-row-reverse' : ''}`}
          >
            {/* Avatar */}
            <div className="shrink-0 mt-5">
              <Facehash name={speakerName} size={32} colors={AVATAR_COLORS} style={{ borderRadius: '25%' }} />
            </div>

            {/* Content */}
            <div className={`flex flex-col max-w-[75%] ${isUser ? 'items-end' : 'items-start'}`}>
              {/* Speaker name + timestamp */}
              <div className={`flex items-center gap-2 mb-1 ${isUser ? 'flex-row-reverse' : ''}`}>
                <span className="relative inline-block">
                  <button
                    onClick={() =>
                      setEditingSpeaker(
                        isEditing ? null : { name: speakerName, segmentId: seg.id }
                      )
                    }
                    className="text-xs font-medium text-gray-500 hover:text-gray-700 transition-colors cursor-pointer focus:outline-none"
                    title="Cliquez pour renommer ce locuteur"
                  >
                    {speakerName}
                  </button>
                  {isEditing && (
                    <SpeakerEditor
                      sessionId={sessionId}
                      currentName={speakerName}
                      onRenamed={(oldName, newName) => {
                        onSpeakerRenamed(oldName, newName);
                        setEditingSpeaker(null);
                      }}
                      onClose={() => setEditingSpeaker(null)}
                    />
                  )}
                </span>
                <span className="text-[10px] text-gray-300 font-mono tabular-nums">
                  {formatTimestamp(seg.start_time)}
                </span>
              </div>

              {/* Chat bubble */}
              <div
                className={`px-4 py-2.5 text-sm leading-relaxed ${
                  isUser
                    ? 'bg-gray-900 text-white rounded-2xl rounded-tr-md'
                    : 'bg-gray-50 border border-gray-100 text-gray-700 rounded-2xl rounded-tl-md'
                }`}
              >
                {seg.text}
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}
