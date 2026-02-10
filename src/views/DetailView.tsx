export default function DetailView() {
  return (
    <div className="max-w-3xl mx-auto space-y-6">
      <h2 className="text-2xl font-bold text-white">Detail de la reunion</h2>

      {/* Loading state */}
      <div className="border border-gray-800 rounded-lg p-8 text-center">
        <p className="text-gray-500">Chargement...</p>
      </div>
    </div>
  );
}
