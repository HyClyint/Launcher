import { useConfig } from '@/context/ConfigContext';
import { tauri } from '@tauri-apps/api';
import { ReactNode, useState } from 'react';
import Modal from './Modal';
import ProfilePicker from './ProfilePicker';

function Row({ title, hint, children }: { title: string; hint: string; children: ReactNode }) {
    return (
        <div className='flex items-center justify-between gap-6 py-5'>
            <div>
                <h3 className='text-sm font-semibold'>{title}</h3>
                <p className='mt-0.5 text-sm text-ink-soft'>{hint}</p>
            </div>
            {children}
        </div>
    );
}

export default function SettingsView() {
    const { config, profile, launch } = useConfig();
    const [picking, setPicking] = useState(false);
    const [confirmReset, setConfirmReset] = useState(false);

    return (
        <section className='h-full overflow-y-auto p-10'>
            <h1 className='text-3xl font-semibold tracking-tight'>Settings</h1>

            <div className='glass anim-rise mt-6 max-w-2xl divide-y divide-white/10 rounded-2xl px-6'>
                <Row title='Account' hint={`Signed in as ${config.username}`}>
                    <button className='btn-ghost' onClick={() => tauri.invoke('logout')} disabled={launch.running}>
                        Sign out
                    </button>
                </Row>
                <Row title='Profiles' hint={`${profile.name} · Minecraft ${profile.game_version}`}>
                    <button className='btn-ghost' onClick={() => setPicking(true)} disabled={launch.running}>
                        Manage
                    </button>
                </Row>
                <Row title='Repair installation' hint='Deletes all game data, profiles and mods, then closes the launcher.'>
                    <button className='btn-danger' onClick={() => setConfirmReset(true)} disabled={launch.running}>
                        Reset
                    </button>
                </Row>
            </div>

            <ProfilePicker open={picking} onClose={() => setPicking(false)} />

            <Modal open={confirmReset} onClose={() => setConfirmReset(false)} title='Reset installation?'>
                <p className='text-sm leading-relaxed text-ink-soft'>This deletes all downloaded game data, every profile, and all mods and worlds, then closes the launcher. You'll stay signed in.</p>
                <div className='mt-5 flex gap-2'>
                    <button className='btn-ghost' onClick={() => setConfirmReset(false)}>
                        Cancel
                    </button>
                    <button
                        className='btn-danger'
                        onClick={async () => {
                            setConfirmReset(false);
                            try {
                                await tauri.invoke('reset');
                            } catch (e) {
                                console.error(e);
                            }
                        }}
                    >
                        Reset
                    </button>
                </div>
            </Modal>
        </section>
    );
}
