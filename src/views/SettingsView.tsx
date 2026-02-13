import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface AudioDevice {
  name: string;
  is_default: boolean;
}

interface SettingsModalProps {
  onClose: () => void;
}

export default function SettingsModal({ onClose }: SettingsModalProps) {
  const [apiKey, setApiKey] = useState('');
  const [inputDevice, setInputDevice] = useState('');
  const [exportDir, setExportDir] = useState('');
  const [devices, setDevices] = useState<AudioDevice[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [showKey, setShowKey] = useState(false);
  const [feedback, setFeedback] = useState<{ type: 'success' | 'error'; message: string } | null>(null);

  // Load all settings on mount
  useEffect(() => {
    (async () => {
      try {
        const [key, deviceSetting, exportSetting, deviceList] = await Promise.all([
          invoke<string>('get_api_key'),
          invoke<string | null>('get_setting', { key: 'input_device' }),
          invoke<string | null>('get_setting', { key: 'export_dir' }),
          invoke<AudioDevice[]>('list_input_devices'),
        ]);
        setApiKey(key);
        setInputDevice(deviceSetting ?? '');
        setExportDir(exportSetting ?? '');
        setDevices(deviceList);
      } catch (err) {
        console.error('Erreur chargement parametres:', err);
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

  // Close on Escape
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'Escape') onClose();
    }
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onClose]);

  const handleSave = useCallback(async () => {
    setSaving(true);
    setFeedback(null);
    try {
      await Promise.all([
        invoke('set_api_key', { key: apiKey }),
        invoke('set_setting', { key: 'input_device', value: inputDevice }),
        invoke('set_setting', { key: 'export_dir', value: exportDir }),
      ]);
      setFeedback({ type: 'success', message: 'Parametres sauvegardes avec succes.' });
    } catch (err) {
      console.error('Erreur sauvegarde parametres:', err);
      setFeedback({ type: 'error', message: `Erreur lors de la sauvegarde: ${err}` });
    } finally {
      setSaving(false);
    }
  }, [apiKey, inputDevice, exportDir]);

  const handlePickFolder = useCallback(async () => {
    try {
      const folder = await invoke<string | null>('pick_folder');
      if (folder) {
        setExportDir(folder);
      }
    } catch (err) {
      console.error('Erreur selection dossier:', err);
    }
  }, []);

  /** Mask the key: show first 4 chars then dots */
  const maskedValue = apiKey.length > 4 ? apiKey.slice(0, 4) + '\u2022'.repeat(Math.min(apiKey.length - 4, 32)) : apiKey;

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 bg-black/20 z-50"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4 pointer-events-none">
        <div
          className="bg-white rounded-2xl shadow-lg border border-gray-100 w-full max-w-lg pointer-events-auto animate-fade-in-down"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Header */}
          <div className="flex items-center justify-between px-6 py-5 border-b border-gray-100">
            <div>
              <h2 className="text-lg font-semibold text-gray-900 tracking-tight">Parametres</h2>
              <p className="text-xs text-gray-400 mt-0.5">Configurez votre application PopTranscribe.</p>
            </div>
            <button
              onClick={onClose}
              className="p-2 rounded-xl text-gray-400 hover:text-gray-600 hover:bg-gray-50 transition-all duration-150"
              title="Fermer"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          {/* Content */}
          <div className="px-6 py-5 space-y-6 max-h-[70vh] overflow-y-auto">
            {loading ? (
              <div className="flex items-center gap-3 py-4">
                <svg className="w-5 h-5 animate-spin text-gray-300" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                </svg>
                <span className="text-sm text-gray-400">Chargement...</span>
              </div>
            ) : (
              <>
                {/* ── API Key section ── */}
                <div>
                  <label htmlFor="api-key" className="block text-sm font-medium text-gray-900 mb-1">
                    Cle API Mistral
                  </label>
                  <p className="text-xs text-gray-400 mb-3">
                    Necessaire pour la transcription et les fonctionnalites IA. Obtenez votre cle sur console.mistral.ai
                  </p>
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
                      className="w-full px-4 py-3 pr-12 bg-gray-50 border border-gray-100 rounded-xl text-sm text-gray-900 placeholder-gray-300 focus:outline-none focus:bg-white focus:border-gray-200 focus:ring-0 transition-all duration-150"
                    />
                    <button
                      type="button"
                      onClick={() => setShowKey(!showKey)}
                      className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-300 hover:text-gray-500 transition-colors p-1"
                      title={showKey ? 'Masquer la cle' : 'Afficher la cle'}
                    >
                      {showKey ? (
                        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={1.5}
                            d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.878 9.878L6.59 6.59m7.532 7.532l3.29 3.29M3 3l18 18"
                          />
                        </svg>
                      ) : (
                        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={1.5}
                            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                          />
                          <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={1.5}
                            d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                          />
                        </svg>
                      )}
                    </button>
                  </div>
                </div>

                {/* ── Microphone section ── */}
                <div>
                  <label htmlFor="input-device" className="block text-sm font-medium text-gray-900 mb-1">
                    Microphone d'entree
                  </label>
                  <p className="text-xs text-gray-400 mb-3">
                    Selectionnez le peripherique audio utilise pour l'enregistrement.
                  </p>
                  <select
                    id="input-device"
                    value={inputDevice}
                    onChange={(e) => setInputDevice(e.target.value)}
                    className="w-full px-4 py-3 bg-gray-50 border border-gray-100 rounded-xl text-sm text-gray-900 focus:outline-none focus:bg-white focus:border-gray-200 focus:ring-0 transition-all duration-150 appearance-none"
                  >
                    <option value="">Par defaut (systeme)</option>
                    {devices.map((device) => (
                      <option key={device.name} value={device.name}>
                        {device.name}{device.is_default ? ' (defaut)' : ''}
                      </option>
                    ))}
                  </select>
                </div>

                {/* ── Export directory section ── */}
                <div>
                  <label className="block text-sm font-medium text-gray-900 mb-1">
                    Dossier d'export
                  </label>
                  <p className="text-xs text-gray-400 mb-3">
                    Choisissez ou les fichiers exportes seront sauvegardes.
                  </p>
                  <div className="flex items-center gap-2">
                    <div className="flex-1 px-4 py-3 bg-gray-50 border border-gray-100 rounded-xl text-sm text-gray-500 truncate">
                      {exportDir || '~/Documents/poptranscribe/exports/ (par defaut)'}
                    </div>
                    <button
                      type="button"
                      onClick={handlePickFolder}
                      className="shrink-0 px-4 py-3 bg-gray-50 border border-gray-100 rounded-xl text-sm text-gray-700 hover:bg-gray-100 transition-all duration-150"
                    >
                      Parcourir...
                    </button>
                    {exportDir && (
                      <button
                        type="button"
                        onClick={() => setExportDir('')}
                        className="shrink-0 p-3 bg-gray-50 border border-gray-100 rounded-xl text-gray-400 hover:text-gray-600 hover:bg-gray-100 transition-all duration-150"
                        title="Reinitialiser au dossier par defaut"
                      >
                        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M6 18L18 6M6 6l12 12" />
                        </svg>
                      </button>
                    )}
                  </div>
                </div>
              </>
            )}

            {/* Save button */}
            <div className="pt-2">
              <button
                onClick={handleSave}
                disabled={loading || saving}
                className={`flex items-center gap-2.5 px-6 py-2.5 bg-gray-900 hover:bg-gray-800 text-white rounded-xl text-sm font-medium transition-all duration-150 shadow-sm ${
                  loading || saving ? 'opacity-50 cursor-not-allowed' : ''
                }`}
              >
                {saving ? (
                  <>
                    <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                    </svg>
                    Sauvegarde...
                  </>
                ) : (
                  'Sauvegarder'
                )}
              </button>
            </div>

            {/* Feedback message */}
            {feedback && (
              <div className={`animate-fade-in-down flex items-center gap-3 px-4 py-3 rounded-xl text-sm ${
                feedback.type === 'success'
                  ? 'bg-emerald-50 text-emerald-700 border border-emerald-100'
                  : 'bg-red-50 text-red-700 border border-red-100'
              }`}>
                {feedback.type === 'success' ? (
                  <svg className="w-4 h-4 text-emerald-400 shrink-0" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
                  </svg>
                ) : (
                  <svg className="w-4 h-4 text-red-400 shrink-0" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v3.75m9-.75a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9 3.75h.008v.008H12v-.008Z" />
                  </svg>
                )}
                {feedback.message}
              </div>
            )}
          </div>
        </div>
      </div>
    </>
  );
}
