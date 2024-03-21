import { useConfig } from '@/context/ConfigContext';
import { Button } from './ui/button';
import { tauri } from '@tauri-apps/api';

type Props = {};

export default function SettingsComponent({}: Props) {
    const { config } = useConfig();

    async function logout() {
        await tauri.invoke('logout');
    }

    async function reset() {
        await tauri.invoke('reset');
    }

    return (
        <div className='bg-first rounded-lg text-white opacity-75 w-full h-full'>
            <ul className='flex w-full h-full flex-col p-4 gap-3'>
                <li className='flex gap-4 w-full h-fit p-4 flex-col rounded-lg border-fifth border-2'>
                    <span className='text-xl'>Logged in as {config.username}</span>
                    <Button variant='destructive' onClick={logout}>
                        Logout
                    </Button>
                </li>
                <li className='flex gap-4 w-full h-fit p-4 flex-col rounded-lg border-fifth border-2'>
                    <span className='text-xl'>Having issues?</span>
                    <Button variant='destructive' onClick={reset}>
                        Reset
                    </Button>
                </li>
            </ul>
        </div>
    );
}
