import type { Summary } from '../types';

interface SummaryPanelProps {
  summary: Summary | null;
  loading?: boolean;
}

export default function SummaryPanel({ summary, loading }: SummaryPanelProps) {
  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center py-16 px-6">
        <svg className="w-6 h-6 animate-spin text-gray-300 mb-3" fill="none" viewBox="0 0 24 24">
          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
        </svg>
        <span className="text-sm text-gray-400">Resume en cours de generation...</span>
      </div>
    );
  }

  if (!summary) {
    return (
      <div className="flex flex-col items-center justify-center py-16 px-6">
        <div className="w-12 h-12 rounded-2xl bg-gray-50 flex items-center justify-center mb-3">
          <svg className="w-6 h-6 text-gray-300" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
          </svg>
        </div>
        <p className="text-sm text-gray-400">Pas de resume disponible</p>
      </div>
    );
  }

  return (
    <div className="space-y-8 p-6 overflow-auto">
      {/* Points cles */}
      <section>
        <h4 className="text-[11px] font-semibold text-gray-400 uppercase tracking-widest mb-4 pb-2 border-b border-gray-100">
          Points cles
        </h4>
        {summary.key_points.length > 0 ? (
          <ul className="space-y-3">
            {summary.key_points.map((point, i) => (
              <li key={i} className="flex gap-3 text-sm text-gray-700 leading-relaxed">
                <span className="text-gray-300 shrink-0 mt-0.5">
                  <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" d="m4.5 12.75 6 6 9-13.5" />
                  </svg>
                </span>
                <span>{point}</span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-sm text-gray-300 italic">Aucun point cle identifie</p>
        )}
      </section>

      {/* Decisions */}
      <section>
        <h4 className="text-[11px] font-semibold text-gray-400 uppercase tracking-widest mb-4 pb-2 border-b border-gray-100">
          Decisions
        </h4>
        {summary.decisions.length > 0 ? (
          <ul className="space-y-3">
            {summary.decisions.map((decision, i) => (
              <li key={i} className="flex gap-3 text-sm text-gray-700 leading-relaxed">
                <span className="text-gray-300 shrink-0 mt-0.5">
                  <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M13.5 4.5 21 12m0 0-7.5 7.5M21 12H3" />
                  </svg>
                </span>
                <span>{decision}</span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-sm text-gray-300 italic">Aucune decision identifiee</p>
        )}
      </section>

      {/* Actions a suivre */}
      <section>
        <h4 className="text-[11px] font-semibold text-gray-400 uppercase tracking-widest mb-4 pb-2 border-b border-gray-100">
          Actions a suivre
        </h4>
        {summary.action_items.length > 0 ? (
          <ul className="space-y-3">
            {summary.action_items.map((item, i) => (
              <li key={i} className="flex gap-3 text-sm text-gray-700 leading-relaxed">
                <span className="shrink-0 mt-0.5">
                  <div className="w-4 h-4 rounded border-2 border-gray-200" />
                </span>
                <span>
                  {item.description}
                  {item.assignee && (
                    <span className="ml-2 inline-flex items-center text-xs font-medium bg-violet-50 text-violet-600 px-2 py-0.5 rounded-md">
                      {item.assignee}
                    </span>
                  )}
                </span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-sm text-gray-300 italic">Aucune action identifiee</p>
        )}
      </section>
    </div>
  );
}
