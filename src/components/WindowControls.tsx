import { appWindow } from '@tauri-apps/api/window';
import { Minus, Square, X } from 'lucide-react';

const btn = 'flex h-8 w-10 items-center justify-center text-ink-soft transition hover:text-ink';

/**
 * Drag region + minimize/maximize/close, drawn inside the UI because the
 * native title bar is disabled. Always on top of every screen, including the
 * splash and sign-in.
 */
export default function WindowControls() {
    return (
        <>
            <div data-tauri-drag-region className='fixed inset-x-0 top-0 z-[70] h-8' onDoubleClick={() => appWindow.toggleMaximize()} />
            <div className='fixed right-2 top-2 z-[71] flex overflow-hidden rounded-lg border border-white/10 bg-black/30 backdrop-blur-md'>
                <button className={`${btn} hover:bg-white/10`} onClick={() => appWindow.minimize()} aria-label='Minimize'>
                    <Minus size={15} />
                </button>
                <button className={`${btn} hover:bg-white/10`} onClick={() => appWindow.toggleMaximize()} aria-label='Maximize'>
                    <Square size={12} />
                </button>
                <button className={`${btn} hover:!bg-danger/80 hover:!text-white`} onClick={() => appWindow.close()} aria-label='Close'>
                    <X size={16} />
                </button>
            </div>
        </>
    );
}
