import { useConfig } from '@/context/ConfigContext';
import { tauri } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import { Check, Download, Loader2, Package, Search, Trash2 } from 'lucide-react';
import { useCallback, useEffect, useRef, useState } from 'react';
import Modal from './Modal';

type ModResult = {
    id: string;
    title: string;
    description: string;
    author: string;
    downloads: number;
    icon_url: string | null;
    installed: boolean;
};

type SearchResults = { results: ModResult[]; total: number };

type InstalledMod = {
    filename: string;
    enabled: boolean;
    title: string;
    icon_url: string | null;
    version_number: string | null;
    project_id: string | null;
    required_by: string[];
};

const compact = new Intl.NumberFormat('en', { notation: 'compact' });

function Icon({ src, size = 'h-12 w-12' }: { src: string | null; size?: string }) {
    return src ? (
        <img src={src} alt='' loading='lazy' className={`${size} shrink-0 rounded-xl bg-black/30 object-cover`} />
    ) : (
        <div className={`${size} flex shrink-0 items-center justify-center rounded-xl bg-white/5 text-ink-mute`}>
            <Package size={20} />
        </div>
    );
}

export default function ModsView() {
    const { profile, launch } = useConfig();
    const [tab, setTab] = useState<'browse' | 'installed'>('browse');
    const [message, setMessage] = useState<{ kind: 'error' | 'info'; text: string } | null>(null);
    const locked = launch.running;

    // Shared between tabs so installing in Browse refreshes Installed.
    const [installed, setInstalled] = useState<InstalledMod[]>([]);
    const refreshInstalled = useCallback(async () => {
        try {
            setInstalled(await tauri.invoke<InstalledMod[]>('list_mods'));
        } catch (e) {
            setMessage({ kind: 'error', text: String(e) });
        }
    }, []);

    useEffect(() => {
        refreshInstalled();
    }, [refreshInstalled, profile.id]);

    return (
        <section className='flex h-full flex-col overflow-hidden p-10 pt-12'>
            <div className='flex items-end justify-between gap-4'>
                <div>
                    <h1 className='text-3xl font-semibold tracking-tight'>Mods</h1>
                    <p className='mt-1 text-sm text-ink-soft'>
                        {profile.name} · Fabric · Minecraft {profile.game_version}
                    </p>
                </div>
                <div className='glass flex rounded-xl p-1' role='tablist'>
                    {(['browse', 'installed'] as const).map((t) => (
                        <button
                            key={t}
                            role='tab'
                            aria-selected={tab === t}
                            onClick={() => setTab(t)}
                            className={`rounded-lg px-4 py-1.5 text-sm font-medium capitalize transition ${tab === t ? 'bg-white/10 text-ink' : 'text-ink-soft hover:text-ink'}`}
                        >
                            {t}
                            {t === 'installed' && installed.length > 0 && <span className='ml-1.5 text-xs text-ink-mute'>{installed.length}</span>}
                        </button>
                    ))}
                </div>
            </div>

            {locked && <p className='mt-4 rounded-xl border border-white/10 bg-black/30 px-4 py-2.5 text-sm text-ink-soft'>Mods can't be changed while the game is starting or running.</p>}
            {message && (
                <p
                    role='alert'
                    onClick={() => setMessage(null)}
                    className={`mt-4 cursor-pointer rounded-xl border px-4 py-2.5 text-sm ${message.kind === 'error' ? 'border-danger/30 bg-danger/10 text-danger' : 'border-white/10 bg-black/30 text-ink-soft'}`}
                >
                    {message.text}
                </p>
            )}

            <div className='mt-5 min-h-0 flex-1'>
                {tab === 'browse' ? (
                    <Browse locked={locked} installed={installed} onChanged={refreshInstalled} onMessage={setMessage} profileId={profile.id} />
                ) : (
                    <Installed locked={locked} mods={installed} onChanged={refreshInstalled} onMessage={setMessage} />
                )}
            </div>
            <p className='mt-3 text-xs text-ink-mute'>Mods are made by third parties via Modrinth. Some can conflict or crash the game.</p>
        </section>
    );
}

type Common = {
    locked: boolean;
    onChanged: () => void;
    onMessage: (m: { kind: 'error' | 'info'; text: string } | null) => void;
};

function Browse({ locked, installed, onChanged, onMessage, profileId }: Common & { installed: InstalledMod[]; profileId: string }) {
    const [query, setQuery] = useState('');
    const [data, setData] = useState<SearchResults | null>(null);
    const [loading, setLoading] = useState(true);
    const [installing, setInstalling] = useState<string | null>(null);
    const [progress, setProgress] = useState('');
    const requestId = useRef(0);

    // Debounced search; stale responses are ignored.
    useEffect(() => {
        const id = ++requestId.current;
        setLoading(true);
        const t = setTimeout(async () => {
            try {
                const result = await tauri.invoke<SearchResults>('search_mods', { query, offset: 0 });
                if (id === requestId.current) {
                    setData(result);
                    onMessage(null);
                }
            } catch (e) {
                if (id === requestId.current) {
                    setData(null);
                    onMessage({ kind: 'error', text: String(e) });
                }
            } finally {
                if (id === requestId.current) setLoading(false);
            }
        }, 300);
        return () => clearTimeout(t);
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [query, profileId]);

    useEffect(() => {
        const unlisten = listen<string>('mod-progress', (e) => setProgress(e.payload));
        return () => {
            unlisten.then((f) => f());
        };
    }, []);

    async function install(mod: ModResult) {
        setInstalling(mod.id);
        onMessage(null);
        try {
            await tauri.invoke('install_mod', { projectId: mod.id });
            onMessage({ kind: 'info', text: `${mod.title} installed.` });
        } catch (e) {
            onMessage({ kind: 'error', text: String(e) });
        } finally {
            setInstalling(null);
            setProgress('');
            onChanged();
        }
    }

    return (
        <div className='flex h-full flex-col'>
            <div className='relative'>
                <Search size={16} className='pointer-events-none absolute left-4 top-1/2 -translate-y-1/2 text-ink-mute' />
                <input className='field !pl-11' placeholder='Search Modrinth' value={query} onChange={(e) => setQuery(e.target.value)} aria-label='Search mods' />
                {loading && <Loader2 size={15} className='absolute right-4 top-1/2 -translate-y-1/2 animate-spin text-ink-mute' />}
            </div>

            <div className='mt-4 min-h-0 flex-1 space-y-2 overflow-y-auto pr-1'>
                {data && data.results.length === 0 && !loading && <p className='py-12 text-center text-sm text-ink-mute'>No Fabric mods found for this version.</p>}
                {data?.results.map((mod) => {
                    const done = mod.installed || installed.some((m) => m.project_id === mod.id);
                    const busy = installing === mod.id;
                    return (
                        <article key={mod.id} className='glass flex items-center gap-4 rounded-2xl p-4'>
                            <Icon src={mod.icon_url} />
                            <div className='min-w-0 flex-1'>
                                <h3 className='truncate text-sm font-semibold'>
                                    {mod.title} <span className='font-normal text-ink-mute'>by {mod.author}</span>
                                </h3>
                                <p className='mt-0.5 line-clamp-2 text-sm text-ink-soft'>{mod.description}</p>
                                <p className='mt-1 flex items-center gap-1 text-xs text-ink-mute'>
                                    <Download size={11} /> {compact.format(mod.downloads)}
                                </p>
                            </div>
                            <button
                                onClick={() => install(mod)}
                                disabled={done || locked || installing !== null}
                                className={done ? 'btn-ghost !w-28' : 'btn-primary !w-28'}
                                title={busy ? progress : undefined}
                            >
                                {busy ? <Loader2 size={15} className='animate-spin' /> : done ? <Check size={15} /> : null}
                                {busy ? 'Installing' : done ? 'Installed' : 'Install'}
                            </button>
                        </article>
                    );
                })}
            </div>
        </div>
    );
}

function Installed({ locked, mods, onChanged, onMessage }: Common & { mods: InstalledMod[] }) {
    const [toRemove, setToRemove] = useState<InstalledMod | null>(null);

    async function toggle(mod: InstalledMod) {
        try {
            await tauri.invoke('set_mod_enabled', { filename: mod.filename, enabled: !mod.enabled });
            onMessage(null);
        } catch (e) {
            onMessage({ kind: 'error', text: String(e) });
        }
        onChanged();
    }

    async function remove() {
        if (!toRemove) return;
        const mod = toRemove;
        setToRemove(null);
        try {
            await tauri.invoke('remove_mod', { filename: mod.filename });
            onMessage(null);
        } catch (e) {
            onMessage({ kind: 'error', text: String(e) });
        }
        onChanged();
    }

    if (mods.length === 0) {
        return (
            <div className='flex h-full flex-col items-center justify-center text-center'>
                <Package size={28} className='text-ink-mute' />
                <p className='mt-3 text-sm text-ink-soft'>No mods in this profile yet.</p>
                <p className='text-sm text-ink-mute'>Find some in the Browse tab.</p>
            </div>
        );
    }

    return (
        <div className='h-full space-y-2 overflow-y-auto pr-1'>
            {mods.map((mod) => (
                <article key={mod.filename} className={`glass flex items-center gap-4 rounded-2xl p-4 transition ${mod.enabled ? '' : 'opacity-60'}`}>
                    <Icon src={mod.icon_url} size='h-10 w-10' />
                    <div className='min-w-0 flex-1'>
                        <h3 className='truncate text-sm font-semibold'>{mod.title}</h3>
                        <p className='truncate text-xs text-ink-mute'>
                            {mod.version_number ? `v${mod.version_number}` : 'Added manually'}
                            {mod.required_by.length > 0 && ` · needed by ${mod.required_by.join(', ')}`}
                        </p>
                    </div>
                    <button
                        role='switch'
                        aria-checked={mod.enabled}
                        aria-label={`${mod.enabled ? 'Disable' : 'Enable'} ${mod.title}`}
                        disabled={locked}
                        onClick={() => toggle(mod)}
                        className={`relative h-6 w-11 shrink-0 rounded-full transition disabled:opacity-50 ${mod.enabled ? 'bg-accent' : 'bg-white/15'}`}
                    >
                        <span className={`absolute top-0.5 h-5 w-5 rounded-full bg-surface transition-all ${mod.enabled ? 'left-[22px]' : 'left-0.5 !bg-ink-soft'}`} />
                    </button>
                    <button
                        aria-label={`Remove ${mod.title}`}
                        disabled={locked}
                        onClick={() => setToRemove(mod)}
                        className='rounded-lg p-2 text-ink-mute transition hover:bg-danger/15 hover:text-danger disabled:opacity-50'
                    >
                        <Trash2 size={16} />
                    </button>
                </article>
            ))}

            <Modal open={toRemove !== null} onClose={() => setToRemove(null)} title='Remove mod?'>
                <p className='text-sm leading-relaxed text-ink-soft'>
                    {toRemove?.title} will be deleted from this profile. You can reinstall it from Browse.
                </p>
                <div className='mt-5 flex gap-2'>
                    <button className='btn-ghost' onClick={() => setToRemove(null)}>
                        Cancel
                    </button>
                    <button className='btn-danger' onClick={remove}>
                        Remove
                    </button>
                </div>
            </Modal>
        </div>
    );
}
