import { X } from 'lucide-react';
import { ReactNode, useEffect } from 'react';

type Props = {
    open: boolean;
    onClose?: () => void;
    title: string;
    children: ReactNode;
    width?: string;
};

/** Centered dialog. Without `onClose` it can't be dismissed (blocking notices). */
export default function Modal({ open, onClose, title, children, width = 'max-w-md' }: Props) {
    useEffect(() => {
        if (!open || !onClose) return;
        const onKey = (e: KeyboardEvent) => e.key === 'Escape' && onClose();
        window.addEventListener('keydown', onKey);
        return () => window.removeEventListener('keydown', onKey);
    }, [open, onClose]);

    if (!open) return null;

    return (
        <div
            className='anim-fade fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-6 backdrop-blur-sm'
            onMouseDown={(e) => e.target === e.currentTarget && onClose?.()}
        >
            <div role='dialog' aria-modal='true' aria-label={title} className={`anim-rise glass w-full ${width} rounded-2xl p-6 shadow-2xl`}>
                <div className='mb-4 flex items-start justify-between gap-4'>
                    <h2 className='text-lg font-semibold tracking-tight'>{title}</h2>
                    {onClose && (
                        <button onClick={onClose} aria-label='Close' className='-m-1 rounded-lg p-1 text-ink-mute transition hover:bg-white/10 hover:text-ink'>
                            <X size={18} />
                        </button>
                    )}
                </div>
                {children}
            </div>
        </div>
    );
}
