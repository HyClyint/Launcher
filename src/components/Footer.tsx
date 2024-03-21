import { event, tauri } from '@tauri-apps/api';
import React, { useEffect } from 'react';

type Props = {};

export default function Footer({}: Props) {
    const [loading, setLoading] = React.useState(false);
    const [text, setText] = React.useState<String>('LAUNCH 1.20.4');

    useEffect(() => {
        let cancelProgress = event.listen<String>('update-progress', (event) => {
            setText(event.payload);
        });

        let cancelGameClosed = event.listen('game-closed', () => {
            setText('LAUNCH 1.20.4');
        });

        return () => {
            cancelProgress.then((c) => c());
            cancelGameClosed.then((c) => c());
        };
    }, []);

    async function onLaunch() {
        setLoading(true);
        await tauri.invoke('launch_client');
        setLoading(false);
        setText('LAUNCH 1.20.4');
    }

    return (
        <div className='bg-first h-[10%] opacity-50 rounded-t-lg text-white flex justify-between'>
            <button disabled={loading} className='px-16 h-full bg-fifth rounded-tl-lg text-4xl font-extrabold'>
                SWITCH VERSION
            </button>
            <button
                onClick={onLaunch}
                disabled={loading}
                className={`px-16 h-full ${loading ? 'bg-red-600' : 'bg-fifth'} rounded-tr-lg text-4xl font-extrabold transition-colors duration-300 ease-in-out`}
            >
                {text}
            </button>
        </div>
    );
}
