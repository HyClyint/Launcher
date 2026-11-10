import { useState } from 'react';
import { useConfig } from '../context/ConfigContext';
import ModsView from './ModsView';
import PlayView from './PlayView';
import SettingsView from './SettingsView';
import Sidebar, { View } from './Sidebar';
import SignIn from './SignIn';

export default function App() {
    const { config } = useConfig();
    const [view, setView] = useState<View>('play');

    return (
        <div className='relative h-full w-full overflow-hidden bg-scene bg-cover bg-center'>
            <div className='absolute inset-0 bg-gradient-to-t from-surface/90 via-surface/40 to-surface/30' aria-hidden />
            <div className='relative flex h-full'>
                {config.is_logged_in ? (
                    <>
                        <Sidebar view={view} onChange={setView} />
                        <main className='min-w-0 flex-1'>{view === 'play' ? <PlayView /> : view === 'mods' ? <ModsView /> : <SettingsView />}</main>
                    </>
                ) : (
                    <div className='h-full w-full'>
                        <SignIn />
                    </div>
                )}
            </div>
        </div>
    );
}
