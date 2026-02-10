import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface SpeakerEditorProps {
  sessionId: string;
  currentName: string;
  onRenamed: (oldName: string, newName: string) => void;
  onClose: () => void;
}

export default function SpeakerEditor({
  sessionId,
  currentName,
  onRenamed,
  onClose,
}: SpeakerEditorProps) {
  const [newName, setNewName] = useState(currentName);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleRename() {
    const trimmed = newName.trim();
    if (!trimmed || trimmed === currentName) {
      onClose();
      return;
    }

    setLoading(true);
    setError(null);

    try {
      await invoke('rename_speaker', {
        sessionId,
        oldName: currentName,
        newName: trimmed,
      });
      onRenamed(currentName, trimmed);
      onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === 'Enter') {
      handleRename();
    } else if (e.key === 'Escape') {
      onClose();
    }
  }

  return (
    <div className="absolute z-50 top-full left-0 mt-1 bg-white border border-gray-200 rounded-xl shadow-lg p-3 min-w-[240px]">
      <label className="block text-xs text-gray-500 mb-1.5">
        Renommer le locuteur
      </label>
      <input
        type="text"
        value={newName}
        onChange={(e) => setNewName(e.target.value)}
        onKeyDown={handleKeyDown}
        autoFocus
        className="w-full px-2.5 py-1.5 bg-white border border-gray-200 rounded text-sm text-gray-900 placeholder-gray-400 focus:outline-none focus:border-gray-400 focus:ring-0"
        placeholder="Nom du locuteur"
      />
      {error && (
        <p className="text-xs text-red-600 mt-1">{error}</p>
      )}
      <div className="flex gap-2 mt-2">
        <button
          onClick={handleRename}
          disabled={loading}
          className="px-3 py-1 bg-gray-900 hover:bg-gray-800 disabled:opacity-50 text-white text-xs rounded font-medium transition-colors"
        >
          {loading ? 'En cours...' : 'Renommer'}
        </button>
        <button
          onClick={onClose}
          disabled={loading}
          className="px-3 py-1 bg-gray-100 hover:bg-gray-200 disabled:opacity-50 text-gray-600 text-xs rounded font-medium transition-colors"
        >
          Annuler
        </button>
      </div>
    </div>
  );
}
