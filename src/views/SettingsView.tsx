import { useState } from 'react';

export default function SettingsView() {
  const [apiKey, setApiKey] = useState('');

  return (
    <div className="max-w-3xl mx-auto space-y-6">
      <h2 className="text-2xl font-bold text-white">Parametres</h2>

      {/* API Key input */}
      <div className="space-y-2">
        <label htmlFor="api-key" className="block text-sm text-gray-400">
          Cle API OpenAI
        </label>
        <input
          id="api-key"
          type="password"
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
          placeholder="sk-..."
          className="w-full px-3 py-2 bg-gray-900 border border-gray-700 rounded-lg text-gray-100 placeholder-gray-600 focus:outline-none focus:border-indigo-500"
        />
      </div>

      {/* Save button */}
      <button className="px-6 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-medium text-sm transition-colors">
        Sauvegarder
      </button>
    </div>
  );
}
