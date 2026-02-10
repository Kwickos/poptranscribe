interface TranscriptLineProps {
  text: string;
  startTime: number;
  highlight?: string;
}

function formatTimestamp(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;
}

function highlightText(text: string, query: string): React.ReactNode {
  if (!query) return text;

  const regex = new RegExp(`(${query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi');
  const parts = text.split(regex);

  return parts.map((part, i) =>
    regex.test(part) ? (
      <mark key={i} className="bg-yellow-400/80 text-gray-950 rounded-sm px-0.5">
        {part}
      </mark>
    ) : (
      part
    )
  );
}

export default function TranscriptLine({ text, startTime, highlight }: TranscriptLineProps) {
  return (
    <div className="flex gap-3 py-1.5 px-2 rounded hover:bg-gray-800/50 transition-colors">
      <span className="text-gray-500 text-sm font-mono shrink-0 pt-0.5">
        [{formatTimestamp(startTime)}]
      </span>
      <span className="text-gray-200 text-sm leading-relaxed">
        {highlight ? highlightText(text, highlight) : text}
      </span>
    </div>
  );
}
