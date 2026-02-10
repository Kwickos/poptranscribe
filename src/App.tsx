import { BrowserRouter, Routes, Route } from 'react-router-dom';
import Layout from './components/Layout';
import SessionView from './views/SessionView';
import HistoryView from './views/HistoryView';
import DetailView from './views/DetailView';
import SettingsView from './views/SettingsView';

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<Layout />}>
          <Route path="/" element={<SessionView />} />
          <Route path="/history" element={<HistoryView />} />
          <Route path="/session/:id" element={<DetailView />} />
          <Route path="/settings" element={<SettingsView />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
