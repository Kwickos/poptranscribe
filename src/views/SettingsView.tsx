import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export default function SettingsView() {
  const [apiKey, setApiKey] = useState('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [showKey, setShowKey] = useState(false);
  const [feedback, setFeedback] = useState<{ type: 'success' | 'error'; message: string } | null>(null);

  // Load the saved API key on mount
  useEffect(() => {
    (async () => {
      try {
        const key = await invoke<string>('get_api_key');
        setApiKey(key);
      } catch (err) {
        console.error('Erreur chargement cle API:', err);
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  // Auto-dismiss feedback after 3 seconds
  useEffect(() => {
    if (!feedback) return;
    const timer = setTimeout(() => setFeedback(null), 3000);
    return () => clearTimeout(timer);
  }, [feedback]);

  const handleSave = useCallback(async () => {
    setSaving(true);
    setFeedback(null);
    try {
      await invoke('set_api_key', { key: apiKey });
      setFeedback({ type: 'success', message: 'Cle API sauvegardee avec succes.' });
    } catch (err) {
      console.error('Erreur sauvegarde cle API:', err);
      setFeedback({ type: 'error', message: `Erreur lors de la sauvegarde: ${err}` });
    } finally {
      setSaving(false);
    }
  }, [apiKey]);

  /** Mask the key: show first 4 chars then dots */
  const maskedValue = apiKey.length > 4 ? apiKey.slice(0, 4) + '\u2022'.repeat(Math.min(apiKey.length - 4, 32)) : apiKey;

  return (
    <div className="max-w-3xl mx-auto space-y-6">
      <h2 className="text-xl font-semibold text-gray-900">Parametres</h2>

      {/* API Key input */}
      <div className="space-y-2">
        <label htmlFor="api-key" className="block text-sm text-gray-500">
          Cle API Mistral
        </label>

        {loading ? (
          <div className="flex items-center gap-2 text-gray-400 text-sm py-2">
            <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
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
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
              />
            </svg>
            Chargement...
          </div>
        ) : (
          <div className="relative">
            <input
              id="api-key"
              type="text"
              value={showKey ? apiKey : maskedValue}
              onChange={(e) => {
                setShowKey(true);
                setApiKey(e.target.value);
              }}
              onFocus={() => setShowKey(true)}
              placeholder="Entrez votre cle API Mistral..."
              className="w-full px-3 py-2 pr-10 bg-white border border-gray-200 rounded-lg text-gray-900 placeholder-gray-400 focus:outline-none focus:border-gray-400 focus:ring-0"
            />
            <button
              type="button"
              onClick={() => setShowKey(!showKey)}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 transition-colors p-1"
              title={showKey ? 'Masquer la cle' : 'Afficher la cle'}
            >
              {showKey ? (
                /* Eye-off icon */
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.878 9.878L6.59 6.59m7.532 7.532l3.29 3.29M3 3l18 18"
                  />
                </svg>
              ) : (
                /* Eye icon */
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                  />
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                  />
                </svg>
              )}
            </button>
          </div>
        )}

        <p className="text-xs text-gray-400">
          Obtenez votre cle sur console.mistral.ai
        </p>
      </div>

      {/* Save button */}
      <button
        onClick={handleSave}
        disabled={loading || saving}
        className={`px-6 py-2 bg-gray-900 hover:bg-gray-800 text-white rounded-lg font-medium text-sm transition-colors ${
          loading || saving ? 'opacity-50 cursor-not-allowed' : ''
        }`}
      >
        {saving ? (
          <span className="flex items-center gap-2">
            <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
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
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
              />
            </svg>
            Sauvegarde...
          </span>
        ) : (
          'Sauvegarder'
        )}
      </button>

      {/* Feedback message */}
      {feedback && (
        <div
          className={`px-4 py-2 rounded-lg text-sm ${
            feedback.type === 'success'
              ? 'bg-emerald-50 text-emerald-700 border border-emerald-200'
              : 'bg-red-50 text-red-700 border border-red-200'
          }`}
        >
          {feedback.message}
        </div>
      )}
    </div>
  );
}
