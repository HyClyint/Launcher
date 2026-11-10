import logo from '@/assets/HyClyint.png';

export default function Splash({ visible }: { visible: boolean }) {
    return (
        <div
            className={`fixed inset-0 z-[60] flex flex-col items-center justify-center gap-6 bg-surface transition-opacity duration-500 ${
                visible ? 'opacity-100' : 'pointer-events-none opacity-0'
            }`}
        >
            <img src={logo} alt='HyClyint' className='h-28 w-28 rounded-3xl shadow-2xl shadow-accent/10' />
            <div className='relative h-1 w-40 overflow-hidden rounded-full bg-white/10'>
                <div className='absolute inset-y-0 w-1/3 rounded-full bg-accent' style={{ animation: 'sheen 1.2s ease-in-out infinite' }} />
            </div>
        </div>
    );
}
