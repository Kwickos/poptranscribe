import { useState } from 'react';

export default function SessionView() {
  const [mode, setMode] = useState<'visio' | 'presentiel'>('visio');

  return (
    <div className="max-w-3xl mx-auto space-y-6">
      <h2 className="text-2xl font-bold text-white">Nouvelle Session</h2>

      {/* Mode toggle */}
      <div className="flex gap-2">
        <button
          onClick={() => setMode('visio')}
          className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
            mode === 'visio'
              ? 'bg-indigo-600 text-white'
              : 'bg-gray-800 text-gray-400 hover:text-white'
          }`}
        >
          Visio
        </button>
        <button
          onClick={() => setMode('presentiel')}
          className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
            mode === 'presentiel'
              ? 'bg-indigo-600 text-white'
              : 'bg-gray-800 text-gray-400 hover:text-white'
          }`}
        >
          Presentiel
        </button>
      </div>

      {/* Start button */}
      <button className="px-6 py-3 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-medium transition-colors">
        Demarrer
      </button>

      {/* Transcript area */}
      <div className="border border-gray-800 rounded-lg p-4 min-h-[300px]">
        <p className="text-gray-500 text-sm">La transcription apparaitra ici...</p>
      </div>
    </div>
  );
}
