import Loading from '@/components/Loading';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent } from '@/components/ui/dialog';
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
    const [updateAvailable, setUpdateAvailable] = useState(false);
    const [updateInfo, setUpdateInfo] = useState<String>('');
    const [failed, setFailed] = useState(false);
    const [error, setError] = useState<String>('');
    const [macosJavaWarning, setMacosJavaWarning] = useState(true);
    const [displayLoading, setDisplayLoading] = useState(true);

    useEffect(() => {
        async function loadConfig() {
            const config: Config = await tauri.invoke('get_config');
            setConfig(config);

            await tauri.invoke('initialize');

            setLoading(false);
        }

        const cancel = listen<Config>('config-updated', (event) => {
            setConfig(event.payload);
            console.log('config updated');
        });

        const cancelUpdate = listen<String>('update-available', (event) => {
            setUpdateAvailable(true);
            setUpdateInfo(event.payload);
        });

        const cancelFailed = listen<String>('update-failed', (event) => {
            setFailed(true);
            setError(event.payload);
        });

        const macosJavaWarning = listen('macos-java-warning', () => {
            setMacosJavaWarning(true);
        });

        loadConfig();

        return () => {
            cancel.then((cancel) => cancel());
            cancelUpdate.then((cancel) => cancel());
            cancelFailed.then((cancel) => cancel());
            macosJavaWarning.then((cancel) => cancel());
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
            {
                <>
                    {children}
                    <Dialog open={updateAvailable}>
                        <DialogContent>
                            <h1 className='scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl text-center'>Update Available</h1>
                            <p className='leading-7 [&:not(:first-child)]:mt-6'>
                                A new update is available. Please join the discord server for more information.
                            </p>
                            <p className='leading-7 [&:not(:first-child)]:mt-6'>{updateInfo}</p>

                            <Button asChild>
                                <a href='https://discord.gg/99zb4wvxzs' target='_blank' rel='noreferrer'>
                                    Join Discord
                                </a>
                            </Button>
                        </DialogContent>
                    </Dialog>
                    <Dialog open={failed}>
                        <DialogContent>
                            <h1 className='scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl text-center'>Something went wrong!</h1>
                            <p className='leading-7 [&:not(:first-child)]:mt-6'>
                                An error occurred while downloading the update. Please join the discord server for more information. Please restart the launcher
                                to try again.
                            </p>
                            <p className='leading-7 [&:not(:first-child)]:mt-6'>{error}</p>

                            <Button asChild>
                                <a href='https://discord.gg/99zb4wvxzs' target='_blank' rel='noreferrer'>
                                    Join Discord
                                </a>
                            </Button>
                        </DialogContent>
                    </Dialog>
                    <Dialog open={macosJavaWarning} onOpenChange={setMacosJavaWarning}>
                        <DialogContent>
                            <h1 className='scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl text-center'>Still under heavy development</h1>
                            <p className='leading-7 [&:not(:first-child)]:mt-6'>
                                On macOS, you need to install Java 21. Please click the button below to download the recommended port of OpenJDK 21 for macOS.
                                Please note that HyClyint is not fully supported on macOS. If you have any issues, please join the discord server and report the
                                issue. Please ignore the warning if you have OpenJDK 21 installed. Thank you!
                            </p>
                            <Button asChild>
                                <a href='https://www.azul.com/downloads/#downloads-table-zulu' target='_blank' rel='noreferrer'>
                                    Download Java 21
                                </a>
                            </Button>
                            <Button asChild>
                                <a href='https://discord.gg/99zb4wvxzs' target='_blank' rel='noreferrer'>
                                    Join Discord
                                </a>
                            </Button>
                        </DialogContent>
                    </Dialog>
                </>
            }
            {displayLoading && <Loading loading={loading} />}
        </ConfigContext.Provider>
    );
}
