import type { Summary } from '../types';

interface SummaryPanelProps {
  summary: Summary | null;
  loading?: boolean;
}

export default function SummaryPanel({ summary, loading }: SummaryPanelProps) {
  if (loading) {
    return (
      <div className="space-y-4 p-4">
        <div className="flex items-center gap-2 text-gray-400 text-sm">
          <svg
            className="animate-spin h-4 w-4"
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
          Resume en cours de generation...
        </div>
      </div>
    );
  }

  if (!summary) {
    return (
      <div className="flex items-center justify-center h-full text-gray-500 text-sm p-4">
        Pas de resume disponible
      </div>
    );
  }

  return (
    <div className="space-y-6 p-4 overflow-auto">
      {/* Points cles */}
      <section>
        <h3 className="text-sm font-semibold text-white mb-2 uppercase tracking-wide">
          Points cles
        </h3>
        {summary.key_points.length > 0 ? (
          <ul className="space-y-1.5">
            {summary.key_points.map((point, i) => (
              <li key={i} className="flex gap-2 text-sm text-gray-300">
                <span className="text-indigo-400 shrink-0">&#8226;</span>
                <span>{point}</span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-sm text-gray-500 italic">Aucun point cle identifie</p>
        )}
      </section>

      {/* Decisions */}
      <section>
        <h3 className="text-sm font-semibold text-white mb-2 uppercase tracking-wide">
          Decisions
        </h3>
        {summary.decisions.length > 0 ? (
          <ul className="space-y-1.5">
            {summary.decisions.map((decision, i) => (
              <li key={i} className="flex gap-2 text-sm text-gray-300">
                <span className="text-emerald-400 shrink-0">&#8226;</span>
                <span>{decision}</span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-sm text-gray-500 italic">Aucune decision identifiee</p>
        )}
      </section>

      {/* Actions a suivre */}
      <section>
        <h3 className="text-sm font-semibold text-white mb-2 uppercase tracking-wide">
          Actions a suivre
        </h3>
        {summary.action_items.length > 0 ? (
          <ul className="space-y-1.5">
            {summary.action_items.map((item, i) => (
              <li key={i} className="flex gap-2 text-sm text-gray-300">
                <span className="text-amber-400 shrink-0">&#8226;</span>
                <span>
                  {item.description}
                  {item.assignee && (
                    <span className="ml-1.5 text-xs bg-gray-800 text-gray-400 px-1.5 py-0.5 rounded">
                      {item.assignee}
                    </span>
                  )}
                </span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-sm text-gray-500 italic">Aucune action identifiee</p>
        )}
      </section>
    </div>
  );
}
