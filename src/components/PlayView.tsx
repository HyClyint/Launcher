import { useConfig } from '@/context/ConfigContext';
import { ChevronDown, Loader2, Play } from 'lucide-react';
import { useState } from 'react';
import ProfilePicker from './ProfilePicker';

export default function PlayView() {
    const { config, profile, launch, startLaunch } = useConfig();
    const [picking, setPicking] = useState(false);

    const label = launch.playing ? 'Playing' : launch.running ? launch.status || 'Preparing…' : 'Play';

    return (
        <section className='relative flex h-full flex-col justify-end p-10'>
            <div className='anim-rise max-w-xl'>
                <p className='text-sm font-medium text-accent'>Welcome back, {config.username}</p>
                <h1 className='mt-2 text-5xl font-semibold leading-[1.05] tracking-tight'>Ready to play?</h1>
                <p className='mt-3 max-w-md text-ink-soft'>HyClyint keeps your game, Fabric and required mods up to date automatically.</p>
            </div>

            <div className='glass mt-8 flex items-stretch gap-3 rounded-2xl p-3'>
                <button
                    onClick={() => setPicking(true)}
                    disabled={launch.running}
                    className='group flex min-w-[170px] flex-col items-start justify-center rounded-xl px-4 py-2 text-left transition hover:bg-white/5 disabled:cursor-not-allowed disabled:opacity-60'
                    aria-label='Change profile'
                >
                    <span className='text-[11px] font-medium uppercase tracking-wider text-ink-mute'>Profile</span>
                    <span className='flex items-center gap-1.5 text-lg font-semibold'>
                        <span className='max-w-[180px] truncate'>{profile.name}</span>
                        <ChevronDown size={16} className='shrink-0 text-ink-mute transition group-hover:translate-y-0.5' />
                    </span>
                    <span className='text-xs text-ink-mute'>Minecraft {profile.game_version}</span>
                </button>

                <button
                    onClick={startLaunch}
                    disabled={launch.running}
                    className='relative flex h-16 flex-1 items-center justify-center gap-3 overflow-hidden rounded-xl bg-accent text-lg font-bold text-surface transition hover:bg-accent-strong disabled:cursor-default disabled:hover:bg-accent'
                >
                    {launch.running && !launch.playing && (
                        <span
                            className='absolute inset-y-0 left-0 bg-black/20 transition-[width] duration-300'
                            style={{ width: `${launch.progress ?? 0}%` }}
                            aria-hidden
                        />
                    )}
                    <span className='relative flex items-center gap-2.5'>
                        {launch.running ? <Loader2 size={20} className={launch.playing ? 'hidden' : 'animate-spin'} /> : <Play size={20} fill='currentColor' />}
                        {label}
                    </span>
                </button>
            </div>

            <ProfilePicker open={picking} onClose={() => setPicking(false)} />
        </section>
    );
}
