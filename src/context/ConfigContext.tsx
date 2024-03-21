import Loading from '@/components/Loading';
import { tauri } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import { createContext, useContext, useEffect, useState } from 'react';

type Config = {
    is_logged_in: boolean;
    username: String;
    selected_version: String;
};

type ConfigContextType = {
    config: Config;
    setConfig: React.Dispatch<React.SetStateAction<Config>>;
};

const ConfigContext = createContext<ConfigContextType>({} as ConfigContextType);

export function useConfig() {
    return useContext(ConfigContext) as ConfigContextType;
}

export function ConfigProvider({ children }: { children: React.ReactNode }) {
    const [config, setConfig] = useState<Config>({
        is_logged_in: false,
        username: '',
        selected_version: '1.20.4',
    });
    const [loading, setLoading] = useState(true);
    const [displayLoading, setDisplayLoading] = useState(true);

    useEffect(() => {
        async function loadConfig() {
            const config: Config = await tauri.invoke('get_config');
            console.log(config);
            setConfig(config);

            await tauri.invoke('initialize');

            setLoading(false);
        }

        const cancel = listen<Config>('config-updated', (event) => {
            setConfig(event.payload);
            console.log('config updated');
        });
        loadConfig();

        return () => {
            cancel.then((cancel) => cancel());
        };
    }, []);

    useEffect(() => {
        if (!loading) {
            const timeout = setTimeout(() => {
                setDisplayLoading(false);
            }, 500);
            return () => clearTimeout(timeout);
        } else {
            setDisplayLoading(true);
        }
    }, [loading]);

    return (
        <ConfigContext.Provider value={{ config, setConfig }}>
            {children}
            {displayLoading && <Loading loading={loading} />}
        </ConfigContext.Provider>
    );
}
