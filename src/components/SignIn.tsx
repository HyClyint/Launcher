import logo from '@/assets/HyClyint.png';
import { tauri } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import { Check, Copy, ExternalLink, Loader2 } from 'lucide-react';
import { FormEvent, useEffect, useState } from 'react';
import Modal from './Modal';

export default function SignIn() {
    const [username, setUsername] = useState('');
    const [busy, setBusy] = useState<'offline' | 'microsoft' | null>(null);
    const [device, setDevice] = useState<{ link: string; code: string } | null>(null);
    const [copied, setCopied] = useState(false);

    const valid = username.trim().length >= 3 && username.trim().length <= 32;

    useEffect(() => {
        const unlisten = [
            listen<string>('microsoft-code', (e) => {
                const [link, code] = e.payload.split(' ');
                setDevice({ link, code });
                setBusy(null);
            }),
            listen('auth-error', () => {
                setBusy(null);
                setDevice(null);
            }),
        ];
        return () => unlisten.forEach((u) => u.then((f) => f()));
    }, []);

    async function offline(e: FormEvent) {
        e.preventDefault();
        if (!valid || busy) return;
        setBusy('offline');
        try {
            await tauri.invoke('login_offline', { username: username.trim() });
        } catch (error) {
            console.error(error); // surfaced by the auth-error dialog
            setBusy(null);
        }
    }

    async function microsoft() {
        setBusy('microsoft');
        try {
            await tauri.invoke('login_microsoft');
        } catch (error) {
            console.error(error);
            setBusy(null);
        }
    }

    async function copy() {
        if (!device) return;
        try {
            await navigator.clipboard.writeText(device.code);
            setCopied(true);
            setTimeout(() => setCopied(false), 1500);
        } catch {
            /* clipboard unavailable; the code is still on screen */
        }
    }

    return (
        <main className='flex h-full items-center justify-center p-6'>
            <div className='anim-rise glass w-full max-w-sm rounded-3xl p-8 shadow-2xl'>
                <img src={logo} alt='' className='mx-auto h-16 w-16 rounded-2xl' />
                <h1 className='mt-5 text-center text-2xl font-semibold tracking-tight'>Welcome to HyClyint</h1>
                <p className='mt-1 text-center text-sm text-ink-soft'>Sign in to start playing.</p>

                <button onClick={microsoft} disabled={busy !== null} className='btn-primary mt-7 w-full'>
                    {busy === 'microsoft' ? <Loader2 size={16} className='animate-spin' /> : null}
                    {busy === 'microsoft' ? 'Getting your code…' : 'Sign in with Microsoft'}
                </button>

                <div className='my-5 flex items-center gap-3 text-xs text-ink-mute'>
                    <span className='h-px flex-1 bg-white/10' />
                    or play offline
                    <span className='h-px flex-1 bg-white/10' />
                </div>

                <form onSubmit={offline} className='space-y-3'>
                    <input
                        className='field'
                        placeholder='Username'
                        value={username}
                        onChange={(e) => setUsername(e.target.value)}
                        maxLength={32}
                        autoComplete='off'
                        spellCheck={false}
                        aria-label='Offline username'
                    />
                    <button type='submit' disabled={!valid || busy !== null} className='btn-ghost w-full'>
                        {busy === 'offline' ? <Loader2 size={16} className='animate-spin' /> : null}
                        Continue offline
                    </button>
                    <p className='text-center text-xs text-ink-mute'>Usernames are 3–32 characters.</p>
                </form>
            </div>

            <Modal open={device !== null} onClose={() => setDevice(null)} title='Finish signing in'>
                <ol className='space-y-3 text-sm text-ink-soft'>
                    <li>1. Open the Microsoft sign-in page.</li>
                    <li>2. Enter this code when asked.</li>
                </ol>
                <div className='mt-4 flex items-center justify-between rounded-xl border border-white/10 bg-black/30 px-4 py-3'>
                    <span className='select-text font-mono text-2xl font-semibold tracking-[0.25em]'>{device?.code}</span>
                    <button onClick={copy} aria-label='Copy code' className='rounded-lg p-2 text-ink-soft transition hover:bg-white/10 hover:text-ink'>
                        {copied ? <Check size={18} className='text-accent' /> : <Copy size={18} />}
                    </button>
                </div>
                <a href={device?.link} target='_blank' rel='noreferrer' className='btn-primary mt-4 w-full'>
                    Open Microsoft <ExternalLink size={15} />
                </a>
                <p className='mt-3 flex items-center justify-center gap-2 text-xs text-ink-mute'>
                    <Loader2 size={12} className='animate-spin' /> Waiting for you to finish…
                </p>
            </Modal>
        </main>
    );
}
