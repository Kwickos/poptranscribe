interface AudioLevelProps {
  level: number; // 0-100
  isActive: boolean;
}

export default function AudioLevel({ level, isActive }: AudioLevelProps) {
  const clampedLevel = Math.max(0, Math.min(100, level));

  // Generate 12 bars for the visualization
  const bars = 12;
  const activeBarCount = Math.round((clampedLevel / 100) * bars);

  return (
    <div className="flex items-center gap-2.5">
      <svg
        className={`w-4 h-4 shrink-0 transition-colors duration-150 ${
          isActive ? 'text-gray-900' : 'text-gray-300'
        }`}
        fill="currentColor"
        viewBox="0 0 24 24"
      >
        <path d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3zm-1-9c0-.55.45-1 1-1s1 .45 1 1v6c0 .55-.45 1-1 1s-1-.45-1-1V5z" />
        <path d="M17 11c0 2.76-2.24 5-5 5s-5-2.24-5-5H5c0 3.53 2.61 6.43 6 6.92V21h2v-3.08c3.39-.49 6-3.39 6-6.92h-2z" />
      </svg>
      <div className="flex items-end gap-[3px] h-5">
        {Array.from({ length: bars }).map((_, i) => (
          <div
            key={i}
            className={`w-[3px] rounded-full transition-all duration-75 ${
              i < activeBarCount && isActive
                ? 'bg-gray-900'
                : 'bg-gray-200'
            }`}
            style={{
              height: `${Math.max(4, ((i + 1) / bars) * 20)}px`,
            }}
          />
        ))}
      </div>
    </div>
  );
}
