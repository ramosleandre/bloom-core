import { createRoot } from 'react-dom/client';
import { App } from './App';
import './styles.css';

// Set transparency before the first React paint, including while the bridge connects.
if (new URLSearchParams(location.search).get('view') === 'pie')
  document.documentElement.dataset.surface = 'pie';

createRoot(document.getElementById('root')!).render(<App />);
