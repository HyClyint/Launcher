import ReactDOM from 'react-dom/client';
import App from './components/App';
import './styles.css';
import WindowControls from './components/WindowControls';
import { ConfigProvider } from './context/ConfigContext';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
    <ConfigProvider>
        <WindowControls />
        <App />
    </ConfigProvider>
);
