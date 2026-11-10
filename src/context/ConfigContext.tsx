import Modal from '@/components/Modal';
import Splash from '@/components/Splash';
import { tauri } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import { createContext, useCallback, useContext, useEffect, useRef, useState } from 'react';

export type Profile = {
    id: string;
    name: string;
    game_version: string;
};

export type Config = {
    is_logged_in: boolean;
    username: string;
    uuid?: string;
    minecraft_token?: string; // only checked for presence, never displayed
    selected_version: string;
    profiles: Profile[];
    selected_profile: string;
};

export type LaunchState = {
    running: boolean; // a launch (download + game session) is in progress
    status: string; // human readable phase
    progress: number | null; // 0-100 when the phase reports a percentage
    playing: boolean; // the game window has been started
};

type ConfigContextType = {
    config: Config;
    /** The profile the next launch uses. */
    profile: Profile;
    launch: LaunchState;
    startLaunch: () => Promise<void>;
};

export const DISCORD = 'https://discord.gg/99zb4wvxzs';
const IDLE: LaunchState = { running: false, status: '', progress: null, playing: false };

const ConfigContext = createContext<ConfigContextType>({} as ConfigContextType);

export function useConfig() {
    return useContext(ConfigContext);
}

function Link({ href, children, primary }: { href: string; children: React.ReactNode; primary?: boolean }) {
    return (
        <a href={href} target='_blank' rel='noreferrer' className={primary ? 'btn-primary' : 'btn-ghost'}>
            {children}
        </a>
    );
}

export function ConfigProvider({ children }: { children: React.ReactNode }) {
    const [config, setConfig] = useState<Config>({ is_logged_in: false, username: '', selected_version: '', profiles: [], selected_profile: '' });
    const [launch, setLaunch] = useState<LaunchState>(IDLE);
    const [loading, setLoading] = useState(true);
    const [showSplash, setShowSplash] = useState(true);
    const [updateInfo, setUpdateInfo] = useState<string | null>(null);
    const [failure, setFailure] = useState<string | null>(null);
    const [authError, setAuthError] = useState<string | null>(null);
    const [macWarning, setMacWarning] = useState(() => /Mac/i.test(navigator.userAgent));
    const launching = useRef(false);

    useEffect(() => {
        const unlisten = [
            listen<Config>('config-updated', (e) => setConfig(e.payload)),
            listen<string>('update-available', (e) => setUpdateInfo(e.payload)),
            listen<string>('update-failed', (e) => {
                setFailure(e.payload);
                setLaunch(IDLE);
            }),
            listen<string>('auth-error', (e) => setAuthError(e.payload)),
            // The backend emits both spellings depending on the phase.
            ...['update-progress', 'update_progress'].map((name) =>
                listen<string>(name, (e) => {
                    const text = e.payload;
                    const pct = text.match(/(\d+(?:\.\d+)?)%/);
                    setLaunch((s) => ({
                        ...s,
                        running: true,
                        status: text,
                        progress: pct ? Math.min(100, Number(pct[1])) : null,
                        playing: s.playing || text === 'Launching',
                    }));
                }),
            ),
            listen('game-closed', () => setLaunch(IDLE)),
        ];

        (async () => {
            try {
                setConfig(await tauri.invoke<Config>('get_config'));
                await tauri.invoke('initialize');
            } finally {
                setLoading(false);
            }
        })();

        return () => unlisten.forEach((u) => u.then((f) => f()));
    }, []);

    useEffect(() => {
        if (loading) return;
        const t = setTimeout(() => setShowSplash(false), 400);
        return () => clearTimeout(t);
    }, [loading]);

    const startLaunch = useCallback(async () => {
        if (launching.current) return;
        launching.current = true;
        setLaunch({ running: true, status: 'Preparing…', progress: null, playing: false });
        try {
            await tauri.invoke('launch_client');
        } catch (error) {
            console.error(error);
        } finally {
            launching.current = false;
            setLaunch(IDLE);
        }
    }, []);

    const profile = config.profiles.find((p) => p.id === config.selected_profile) ??
        config.profiles[0] ?? { id: '', name: 'Default', game_version: config.selected_version };

    return (
        <ConfigContext.Provider value={{ config, profile, launch, startLaunch }}>
            {children}

            <Modal open={updateInfo !== null} title='Update available'>
                <p className='text-sm leading-relaxed text-ink-soft'>
                    A new version of the launcher is required to keep playing. Grab it from our Discord.
                </p>
                {updateInfo && <p className='mt-3 rounded-lg bg-black/30 p-3 text-xs text-ink-mute'>{updateInfo}</p>}
                <div className='mt-5 flex'>
                    <Link href={DISCORD} primary>
                        Open Discord
                    </Link>
                </div>
            </Modal>

            <Modal open={failure !== null} onClose={() => setFailure(null)} title='Something went wrong'>
                <p className='text-sm leading-relaxed text-ink-soft'>
                    {failure}. Check your connection and try again. If it keeps happening, let us know on Discord.
                </p>
                <div className='mt-5 flex gap-2'>
                    <button className='btn-ghost' onClick={() => setFailure(null)}>
                        Dismiss
                    </button>
                    <Link href={DISCORD} primary>
                        Get help
                    </Link>
                </div>
            </Modal>

            <Modal open={authError !== null} onClose={() => setAuthError(null)} title='Access denied'>
                <p className='text-sm leading-relaxed text-ink-soft'>{authError}</p>
                <p className='mt-3 text-xs text-ink-mute'>Think this is a mistake? Open a ticket on Discord.</p>
                <div className='mt-5 flex gap-2'>
                    <button className='btn-ghost' onClick={() => setAuthError(null)}>
                        Close
                    </button>
                    <Link href={DISCORD} primary>
                        Open ticket
                    </Link>
                </div>
            </Modal>

            <Modal open={macWarning} onClose={() => setMacWarning(false)} title='Java required on macOS'>
                <p className='text-sm leading-relaxed text-ink-soft'>
                    macOS support is still in development. Install the Java version your Minecraft version needs yourself: Java 25 for 26.x, Java 21 for 1.20.5 to 1.21.x. You can skip this if you already have it.
                </p>
                <div className='mt-5 flex gap-2'>
                    <button className='btn-ghost' onClick={() => setMacWarning(false)}>
                        I have it
                    </button>
                    <Link href='https://www.azul.com/downloads/#downloads-table-zulu' primary>
                        Download Java
                    </Link>
                </div>
            </Modal>

            <Splash visible={showSplash} />
        </ConfigContext.Provider>
    );
}
