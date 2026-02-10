export default function HistoryView() {
  return (
    <div className="max-w-3xl mx-auto space-y-6">
      <h2 className="text-2xl font-bold text-white">Historique</h2>

      {/* Empty state */}
      <div className="border border-gray-800 rounded-lg p-8 text-center">
        <p className="text-gray-500">Aucune reunion enregistree</p>
      </div>
    </div>
  );
}
