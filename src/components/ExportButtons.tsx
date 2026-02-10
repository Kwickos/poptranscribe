import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface ExportButtonsProps {
  sessionId: string;
}

type ExportFormat = 'markdown' | 'pdf' | 'notion' | 'slack';

interface ExportOption {
  format: ExportFormat;
  label: string;
  icon: React.ReactNode;
}

const EXPORT_OPTIONS: ExportOption[] = [
  {
    format: 'markdown',
    label: 'Markdown',
    icon: (
      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
      </svg>
    ),
  },
  {
    format: 'pdf',
    label: 'PDF',
    icon: (
      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
      </svg>
    ),
  },
  {
    format: 'notion',
    label: 'Notion',
    icon: (
      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
      </svg>
    ),
  },
  {
    format: 'slack',
    label: 'Slack',
    icon: (
      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
      </svg>
    ),
  },
];

interface Toast {
  id: number;
  message: string;
  type: 'success' | 'error';
}

export default function ExportButtons({ sessionId }: ExportButtonsProps) {
  const [loadingFormat, setLoadingFormat] = useState<ExportFormat | null>(null);
  const [toasts, setToasts] = useState<Toast[]>([]);

  function addToast(message: string, type: 'success' | 'error') {
    const id = Date.now();
    setToasts((prev) => [...prev, { id, message, type }]);
    setTimeout(() => {
      setToasts((prev) => prev.filter((t) => t.id !== id));
    }, 3000);
  }

  async function handleExport(format: ExportFormat) {
    setLoadingFormat(format);
    try {
      await invoke('export_session', { sessionId, format });
      addToast(`Export ${format.toUpperCase()} reussi`, 'success');
    } catch (err) {
      addToast(`Erreur d'export : ${String(err)}`, 'error');
    } finally {
      setLoadingFormat(null);
    }
  }

  return (
    <>
      <div className="flex items-center gap-2">
        <span className="text-xs text-gray-500 mr-1">Exporter :</span>
        {EXPORT_OPTIONS.map((opt) => (
          <button
            key={opt.format}
            onClick={() => handleExport(opt.format)}
            disabled={loadingFormat !== null}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-gray-800 hover:bg-gray-700 disabled:opacity-50 text-gray-300 text-xs rounded-lg font-medium transition-colors"
          >
            {loadingFormat === opt.format ? (
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
            ) : (
              opt.icon
            )}
            {opt.label}
          </button>
        ))}
      </div>

      {/* Toast notifications */}
      {toasts.length > 0 && (
        <div className="fixed bottom-6 right-6 z-50 flex flex-col gap-2">
          {toasts.map((toast) => (
            <div
              key={toast.id}
              className={`px-4 py-2.5 rounded-lg text-sm font-medium shadow-lg animate-[fadeIn_0.2s_ease-out] ${
                toast.type === 'success'
                  ? 'bg-emerald-600 text-white'
                  : 'bg-red-600 text-white'
              }`}
            >
              {toast.message}
            </div>
          ))}
        </div>
      )}
    </>
  );
}
