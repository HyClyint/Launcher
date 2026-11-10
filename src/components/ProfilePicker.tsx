import { Profile, useConfig } from '@/context/ConfigContext';
import { tauri } from '@tauri-apps/api';
import { ArrowLeft, Check, Loader2, Plus, Sparkles, Trash2 } from 'lucide-react';
import { FormEvent, useEffect, useState } from 'react';
import Modal from './Modal';

type GameVersion = { id: string; installed: boolean };
type Featured = { name: string; game_version: string; hyclyint_version: string; channel: string; exists: boolean };

type Props = {
    open: boolean;
    onClose: () => void;
};

export default function ProfilePicker({ open, onClose }: Props) {
    const { config } = useConfig();
    const [creating, setCreating] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [busy, setBusy] = useState(false);
    const [toDelete, setToDelete] = useState<Profile | null>(null);
    const [featured, setFeatured] = useState<Featured | null>(null);

    useEffect(() => {
        if (open) {
            setCreating(false);
            setError(null);
            setFeatured(null);
            // Optional extra: if the backend is unreachable we simply show nothing.
            tauri
                .invoke<Featured>('get_featured_profile')
                .then(setFeatured)
                .catch(() => {});
        }
    }, [open, config.profiles.length]);

    async function run(action: () => Promise<unknown>, done?: () => void) {
        setBusy(true);
        setError(null);
        try {
            await action();
            done?.();
        } catch (e) {
            setError(String(e));
        } finally {
            setBusy(false);
        }
    }

    return (
        <Modal open={open} onClose={onClose} title={creating ? 'New profile' : 'Profiles'} width='max-w-sm'>
            {creating ? (
                <NewProfile
                    busy={busy}
                    onBack={() => setCreating(false)}
                    onCreate={(name, version) =>
                        run(
                            async () => {
                                const id = await tauri.invoke<string>('create_profile', { name, gameVersion: version });
                                await tauri.invoke('select_profile', { id });
                            },
                            onClose,
                        )
                    }
                />
            ) : (
                <>
                    {featured && !featured.exists && (
                        <button
                            disabled={busy}
                            onClick={() =>
                                run(async () => {
                                    const id = await tauri.invoke<string>('create_profile', { name: featured.name, gameVersion: featured.game_version });
                                    await tauri.invoke('select_profile', { id });
                                }, onClose)
                            }
                            className='mb-3 flex w-full items-center gap-3 rounded-xl border border-accent/30 bg-accent/10 px-3 py-3 text-left transition hover:bg-accent/15 disabled:opacity-60'
                        >
                            <Sparkles size={18} className='shrink-0 text-accent' />
                            <span className='min-w-0 flex-1'>
                                <span className='block text-sm font-semibold'>
                                    {featured.name}
                                    {featured.channel && <span className='ml-2 rounded-md bg-white/10 px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-ink-soft'>{featured.channel}</span>}
                                </span>
                                <span className='block text-xs text-ink-soft'>Newest HyClyint build · v{featured.hyclyint_version}</span>
                            </span>
                            <span className='text-xs font-medium text-accent'>Add</span>
                        </button>
                    )}
                    <div className='-mx-1 max-h-72 space-y-1 overflow-y-auto px-1' role='listbox' aria-label='Profiles'>
                        {config.profiles.map((p) => {
                            const active = p.id === config.selected_profile;
                            return (
                                <div
                                    key={p.id}
                                    className={`group flex items-center rounded-xl transition ${active ? 'bg-accent/10 ring-1 ring-accent/40' : 'hover:bg-white/5'}`}
                                >
                                    <button
                                        role='option'
                                        aria-selected={active}
                                        disabled={busy}
                                        onClick={() => run(() => tauri.invoke('select_profile', { id: p.id }), onClose)}
                                        className='flex min-w-0 flex-1 items-center gap-3 px-3 py-2.5 text-left'
                                    >
                                        <span className='min-w-0 flex-1'>
                                            <span className='block truncate text-sm font-medium'>{p.name}</span>
                                            <span className='block text-xs text-ink-mute'>Minecraft {p.game_version}</span>
                                        </span>
                                        {active && <Check size={16} className='text-accent' />}
                                    </button>
                                    {p.id !== 'default' && (
                                        <button
                                            aria-label={`Delete ${p.name}`}
                                            disabled={busy}
                                            onClick={() => setToDelete(p)}
                                            className='mr-2 rounded-lg p-2 text-ink-mute opacity-0 transition hover:bg-danger/15 hover:text-danger focus-visible:opacity-100 group-hover:opacity-100'
                                        >
                                            <Trash2 size={15} />
                                        </button>
                                    )}
                                </div>
                            );
                        })}
                    </div>
                    <button onClick={() => setCreating(true)} className='btn-ghost mt-3 w-full'>
                        <Plus size={16} /> New profile
                    </button>
                    <p className='mt-3 text-xs text-ink-mute'>Each profile has its own worlds, settings and mods.</p>
                </>
            )}
            {error && <p className='mt-3 text-sm text-danger'>{error}</p>}

            <Modal open={toDelete !== null} onClose={() => setToDelete(null)} title='Delete profile?'>
                <p className='text-sm leading-relaxed text-ink-soft'>
                    {toDelete?.name} and everything in it (worlds, settings and mods) will be permanently deleted.
                </p>
                <div className='mt-5 flex gap-2'>
                    <button className='btn-ghost' onClick={() => setToDelete(null)}>
                        Cancel
                    </button>
                    <button
                        className='btn-danger'
                        onClick={() => {
                            const target = toDelete;
                            setToDelete(null);
                            if (target) run(() => tauri.invoke('delete_profile', { id: target.id }));
                        }}
                    >
                        Delete
                    </button>
                </div>
            </Modal>
        </Modal>
    );
}

function NewProfile({ busy, onBack, onCreate }: { busy: boolean; onBack: () => void; onCreate: (name: string, version: string) => void }) {
    const [name, setName] = useState('');
    const [versions, setVersions] = useState<GameVersion[] | null>(null);
    const [version, setVersion] = useState('');
    const [loadError, setLoadError] = useState<string | null>(null);

    useEffect(() => {
        tauri
            .invoke<GameVersion[]>('get_versions')
            .then((v) => {
                setVersions(v);
                setVersion(v[0]?.id ?? '');
            })
            .catch((e) => setLoadError(String(e)));
    }, []);

    const valid = name.trim().length > 0 && version !== '';

    function submit(e: FormEvent) {
        e.preventDefault();
        if (valid && !busy) onCreate(name.trim(), version);
    }

    return (
        <form onSubmit={submit} className='space-y-3'>
            <input autoFocus className='field' placeholder='Profile name' maxLength={32} value={name} onChange={(e) => setName(e.target.value)} aria-label='Profile name' />
            {loadError ? (
                <p className='text-sm text-danger'>Couldn't load versions: {loadError}</p>
            ) : versions === null ? (
                <p className='flex items-center gap-2 text-sm text-ink-mute'>
                    <Loader2 size={14} className='animate-spin' /> Loading versions…
                </p>
            ) : (
                <label className='block'>
                    <span className='mb-1.5 block text-xs font-medium text-ink-mute'>Minecraft version</span>
                    <select className='field' value={version} onChange={(e) => setVersion(e.target.value)}>
                        {versions.map((v, i) => (
                            <option key={v.id} value={v.id} className='bg-surface'>
                                {v.id}
                                {i === 0 ? ' (latest)' : ''}
                                {v.installed ? ' · installed' : ''}
                            </option>
                        ))}
                    </select>
                </label>
            )}
            <div className='flex gap-2 pt-1'>
                <button type='button' onClick={onBack} className='btn-ghost' aria-label='Back'>
                    <ArrowLeft size={16} />
                </button>
                <button type='submit' disabled={!valid || busy} className='btn-primary flex-1'>
                    {busy && <Loader2 size={16} className='animate-spin' />}
                    Create profile
                </button>
            </div>
        </form>
    );
}
