import { useState, useMemo } from 'react';
import type { Segment } from '../types';
import SpeakerEditor from './SpeakerEditor';

const SPEAKER_COLORS = [
  'text-gray-900',
  'text-gray-600',
  'text-blue-700',
  'text-emerald-700',
  'text-amber-700',
  'text-violet-700',
  'text-rose-700',
  'text-teal-700',
];

function getSpeakerColor(speaker: string, speakers: string[]): string {
  const index = speakers.indexOf(speaker);
  return SPEAKER_COLORS[index % SPEAKER_COLORS.length];
}

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

  if (segments.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-gray-400 text-sm">
        Aucun segment de transcription disponible.
      </div>
    );
  }

  return (
    <div className="space-y-0.5 overflow-auto">
      {segments.map((seg) => {
        const speakerName = seg.speaker ?? 'Inconnu';
        const color = getSpeakerColor(speakerName, speakers);
        const isEditing =
          editingSpeaker !== null &&
          editingSpeaker.segmentId === seg.id;

        return (
          <div
            key={seg.id}
            className="flex gap-3 py-1.5 px-2 rounded hover:bg-gray-50 transition-colors"
          >
            <span className="text-gray-400 text-sm font-mono shrink-0 pt-0.5">
              [{formatTimestamp(seg.start_time)}]
            </span>
            <div className="flex-1 text-sm leading-relaxed">
              <span className="relative inline-block">
                <button
                  onClick={() =>
                    setEditingSpeaker(
                      isEditing ? null : { name: speakerName, segmentId: seg.id }
                    )
                  }
                  className={`${color} font-medium hover:underline cursor-pointer focus:outline-none`}
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
              <span className="text-gray-300 mx-1">:</span>
              <span className="text-gray-700">{seg.text}</span>
            </div>
          </div>
        );
      })}
    </div>
  );
}
