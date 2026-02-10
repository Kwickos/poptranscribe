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
    <div className="absolute z-50 top-full left-0 mt-1 bg-gray-800 border border-gray-700 rounded-lg shadow-xl p-3 min-w-[240px]">
      <label className="block text-xs text-gray-400 mb-1.5">
        Renommer le locuteur
      </label>
      <input
        type="text"
        value={newName}
        onChange={(e) => setNewName(e.target.value)}
        onKeyDown={handleKeyDown}
        autoFocus
        className="w-full px-2.5 py-1.5 bg-gray-900 border border-gray-700 rounded text-sm text-gray-100 placeholder-gray-500 focus:outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
        placeholder="Nom du locuteur"
      />
      {error && (
        <p className="text-xs text-red-400 mt-1">{error}</p>
      )}
      <div className="flex gap-2 mt-2">
        <button
          onClick={handleRename}
          disabled={loading}
          className="px-3 py-1 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white text-xs rounded font-medium transition-colors"
        >
          {loading ? 'En cours...' : 'Renommer'}
        </button>
        <button
          onClick={onClose}
          disabled={loading}
          className="px-3 py-1 bg-gray-700 hover:bg-gray-600 disabled:opacity-50 text-gray-300 text-xs rounded font-medium transition-colors"
        >
          Annuler
        </button>
      </div>
    </div>
  );
}
