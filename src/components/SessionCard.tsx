import type { Session } from '../types';

interface SessionCardProps {
  session: Session;
  onClick: () => void;
}

function formatDuration(secs: number): string {
  const hours = Math.floor(secs / 3600);
  const minutes = Math.floor((secs % 3600) / 60);

  if (hours > 0) {
    return `${hours}h ${minutes.toString().padStart(2, '0')}min`;
  }
  return `${minutes}min`;
}

function formatDate(isoString: string): string {
  const date = new Date(isoString);

  const days = ['Dimanche', 'Lundi', 'Mardi', 'Mercredi', 'Jeudi', 'Vendredi', 'Samedi'];
  const months = [
    'jan.', 'fev.', 'mars', 'avr.', 'mai', 'juin',
    'juil.', 'aout', 'sept.', 'oct.', 'nov.', 'dec.',
  ];

  const dayName = days[date.getDay()];
  const dayNum = date.getDate();
  const month = months[date.getMonth()];
  const year = date.getFullYear();
  const hours = date.getHours().toString().padStart(2, '0');
  const minutes = date.getMinutes().toString().padStart(2, '0');

  return `${dayName} ${dayNum} ${month} ${year}, ${hours}:${minutes}`;
}

function ModeIcon({ mode }: { mode: string }) {
  if (mode === 'visio') {
    return (
      <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-600">
        <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" d="m15.75 10.5 4.72-4.72a.75.75 0 0 1 1.28.53v11.38a.75.75 0 0 1-1.28.53l-4.72-4.72M4.5 18.75h9a2.25 2.25 0 0 0 2.25-2.25v-9a2.25 2.25 0 0 0-2.25-2.25h-9A2.25 2.25 0 0 0 2.25 7.5v9a2.25 2.25 0 0 0 2.25 2.25Z" />
        </svg>
        Visio
      </span>
    );
  }

  return (
    <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-600">
      <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" d="M12 18.75a6 6 0 0 0 6-6v-1.5m-6 7.5a6 6 0 0 1-6-6v-1.5m6 7.5v3.75m-3.75 0h7.5M12 15.75a3 3 0 0 1-3-3V4.5a3 3 0 1 1 6 0v8.25a3 3 0 0 1-3 3Z" />
      </svg>
      Presentiel
    </span>
  );
}

export default function SessionCard({ session, onClick }: SessionCardProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="w-full text-left bg-white border border-gray-200 rounded-xl p-4 hover:border-gray-300 hover:shadow-sm transition-all cursor-pointer"
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <h3 className="text-sm font-medium text-gray-900 truncate">
            {session.title}
          </h3>
          <p className="mt-1 text-xs text-gray-400">
            {formatDate(session.created_at)}
          </p>
        </div>
        <div className="flex flex-col items-end gap-2 shrink-0">
          <ModeIcon mode={session.mode} />
          {session.duration_secs != null && (
            <span className="text-xs text-gray-400">
              {formatDuration(session.duration_secs)}
            </span>
          )}
        </div>
      </div>
    </button>
  );
}
