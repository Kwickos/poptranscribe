import { NavLink, Outlet } from 'react-router-dom';

export default function Layout() {
  return (
    <div className="flex h-screen bg-gray-950 text-gray-100">
      {/* Sidebar */}
      <nav className="w-56 border-r border-gray-800 p-4 flex flex-col gap-2">
        <h1 className="text-lg font-bold mb-4 text-white">PopTranscribe</h1>
        <NavLink to="/" className={({ isActive }) =>
          `px-3 py-2 rounded-lg text-sm ${isActive ? 'bg-indigo-600 text-white' : 'text-gray-400 hover:text-white hover:bg-gray-800'}`
        } end>
          Session
        </NavLink>
        <NavLink to="/history" className={({ isActive }) =>
          `px-3 py-2 rounded-lg text-sm ${isActive ? 'bg-indigo-600 text-white' : 'text-gray-400 hover:text-white hover:bg-gray-800'}`
        }>
          Historique
        </NavLink>
        <div className="flex-1" />
        <NavLink to="/settings" className={({ isActive }) =>
          `px-3 py-2 rounded-lg text-sm ${isActive ? 'bg-indigo-600 text-white' : 'text-gray-400 hover:text-white hover:bg-gray-800'}`
        }>
          Parametres
        </NavLink>
      </nav>
      {/* Main content */}
      <main className="flex-1 overflow-auto p-6">
        <Outlet />
      </main>
    </div>
  );
}
