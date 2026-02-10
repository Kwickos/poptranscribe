interface AudioLevelProps {
  level: number; // 0-100
  isActive: boolean;
}

export default function AudioLevel({ level, isActive }: AudioLevelProps) {
  const clampedLevel = Math.max(0, Math.min(100, level));

  return (
    <div className="flex items-center gap-2">
      <svg
        className={`w-4 h-4 shrink-0 ${isActive ? 'text-green-400' : 'text-gray-600'}`}
        fill="currentColor"
        viewBox="0 0 24 24"
      >
        <path d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3zm-1-9c0-.55.45-1 1-1s1 .45 1 1v6c0 .55-.45 1-1 1s-1-.45-1-1V5z" />
        <path d="M17 11c0 2.76-2.24 5-5 5s-5-2.24-5-5H5c0 3.53 2.61 6.43 6 6.92V21h2v-3.08c3.39-.49 6-3.39 6-6.92h-2z" />
      </svg>
      <div className="w-24 h-2 bg-gray-800 rounded-full overflow-hidden">
        <div
          className={`h-full rounded-full transition-all duration-75 ${
            isActive ? 'bg-green-500' : 'bg-gray-600'
          }`}
          style={{ width: `${clampedLevel}%` }}
        />
      </div>
    </div>
  );
}
