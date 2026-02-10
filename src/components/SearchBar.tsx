import { useState, useCallback } from 'react';

interface SearchBarProps {
  onTextSearch: (query: string) => void;
  onLlmSearch: (query: string) => void;
  isSearching: boolean;
}

export default function SearchBar({ onTextSearch, onLlmSearch, isSearching }: SearchBarProps) {
  const [value, setValue] = useState('');

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const newValue = e.target.value;
      setValue(newValue);

      // Live text search (only if not an LLM query)
      if (!newValue.startsWith('?')) {
        onTextSearch(newValue);
      }
    },
    [onTextSearch]
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === 'Enter') {
        if (value.startsWith('?')) {
          const query = value.slice(1).trim();
          if (query) {
            onLlmSearch(query);
          }
        } else {
          onTextSearch(value);
        }
      }
    },
    [value, onTextSearch, onLlmSearch]
  );

  const handleLlmClick = useCallback(() => {
    const query = value.startsWith('?') ? value.slice(1).trim() : value.trim();
    if (query) {
      onLlmSearch(query);
    }
  }, [value, onLlmSearch]);

  return (
    <div className="flex items-center gap-3">
      <div className="relative flex-1">
        <svg
          className="absolute left-4 top-1/2 -translate-y-1/2 w-4.5 h-4.5 text-gray-300"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={1.5}
            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
          />
        </svg>
        <input
          type="text"
          value={value}
          onChange={handleChange}
          onKeyDown={handleKeyDown}
          placeholder="Rechercher... (prefixer avec ? pour une recherche IA)"
          className="w-full bg-gray-50 border border-gray-100 rounded-xl py-3 pl-11 pr-4 text-sm text-gray-900 placeholder-gray-300 focus:outline-none focus:bg-white focus:border-gray-200 focus:ring-0 transition-all duration-150"
        />
      </div>
      <button
        onClick={handleLlmClick}
        disabled={isSearching || !value.trim()}
        className="flex items-center gap-2 px-4 py-3 bg-gray-900 hover:bg-gray-800 disabled:bg-gray-100 disabled:text-gray-300 text-white text-sm font-medium rounded-xl transition-all duration-150 shrink-0"
        title="Recherche IA"
      >
        {isSearching ? (
          <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
          </svg>
        ) : (
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={1.5}
              d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"
            />
          </svg>
        )}
        IA
      </button>
    </div>
  );
}
