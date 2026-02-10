import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useNavigate } from 'react-router-dom';
import type { Session } from '../types';
import SessionCard from '../components/SessionCard';

export default function HistoryView() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [search, setSearch] = useState('');
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();

  useEffect(() => {
    invoke<Session[]>('get_sessions')
      .then(setSessions)
      .catch(() => setSessions([]))
      .finally(() => setLoading(false));
  }, []);

  const filtered = sessions.filter((s) =>
    s.title.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className="max-w-3xl mx-auto space-y-6">
      <h2 className="text-2xl font-bold text-white">Historique</h2>

      {/* Search bar */}
      <div className="relative">
        <svg
          className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500"
          fill="none"
          viewBox="0 0 24 24"
          strokeWidth={1.5}
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z"
          />
        </svg>
        <input
          type="text"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Rechercher une reunion..."
          className="w-full pl-10 pr-3 py-2 bg-gray-900 border border-gray-700 rounded-lg text-gray-100 placeholder-gray-600 text-sm focus:outline-none focus:border-indigo-500 transition-colors"
        />
      </div>

      {/* Content */}
      {loading ? (
        <div className="border border-gray-800 rounded-lg p-8 text-center">
          <p className="text-gray-500">Chargement...</p>
        </div>
      ) : filtered.length === 0 ? (
        <div className="border border-gray-800 rounded-lg p-8 text-center">
          <p className="text-gray-500">
            {sessions.length === 0
              ? 'Aucune reunion enregistree'
              : 'Aucun resultat pour cette recherche'}
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {filtered.map((session) => (
            <SessionCard
              key={session.id}
              session={session}
              onClick={() => navigate(`/session/${session.id}`)}
            />
          ))}
        </div>
      )}
    </div>
  );
}
