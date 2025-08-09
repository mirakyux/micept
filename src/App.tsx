import { useAppState } from './hooks/useAppState';
import { StatusBar } from './components';
import './App.css';

function App() {
  const { appState } = useAppState();

  return (
    <div className="app-container">
      <StatusBar appState={appState} />
    </div>
  );
}

export default App;
