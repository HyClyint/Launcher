import logo from '@/assets/HyClyint.png';
import Avatar from './Avatar';
import { DISCORD, useConfig } from '@/context/ConfigContext';
import { LucideIcon, MessageCircle, Package, Play, Settings } from 'lucide-react';

export type View = 'play' | 'mods' | 'settings';

const items: { id: View; label: string; icon: LucideIcon }[] = [
    { id: 'play', label: 'Play', icon: Play },
    { id: 'mods', label: 'Mods', icon: Package },
    { id: 'settings', label: 'Settings', icon: Settings },
];

const base = 'flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-sm font-medium transition';

export default function Sidebar({ view, onChange }: { view: View; onChange: (v: View) => void }) {
    const { config, profile } = useConfig();

    return (
        <aside className='glass flex w-60 shrink-0 flex-col border-y-0 border-l-0 p-4'>
            <div className='flex items-center gap-3 px-2 py-2'>
                <img src={logo} alt='' className='h-9 w-9 rounded-xl' />
                <span className='text-lg font-semibold tracking-tight'>HyClyint</span>
            </div>

            <nav className='mt-6 space-y-1'>
                {items.map(({ id, label, icon: Icon }) => (
                    <button
                        key={id}
                        onClick={() => onChange(id)}
                        aria-current={view === id ? 'page' : undefined}
                        className={`${base} ${view === id ? 'bg-white/10 text-ink' : 'text-ink-soft hover:bg-white/5 hover:text-ink'}`}
                    >
                        <Icon size={17} className={view === id ? 'text-accent' : ''} />
                        {label}
                    </button>
                ))}
                <a href={DISCORD} target='_blank' rel='noreferrer' className={`${base} text-ink-soft hover:bg-white/5 hover:text-ink`}>
                    <MessageCircle size={17} />
                    Discord
                </a>
            </nav>

            <div className='mt-auto flex items-center gap-3 rounded-xl bg-black/20 p-3'>
                <Avatar />
                <div className='min-w-0'>
                    <p className='truncate text-sm font-medium'>{config.username}</p>
                    <p className='text-xs text-ink-mute'>{profile.name} · {profile.game_version}</p>
                </div>
            </div>
        </aside>
    );
}
