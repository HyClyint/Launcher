import { useConfig } from '@/context/ConfigContext';
import { useEffect, useState } from 'react';

/** Player head for Microsoft accounts; initial letter for offline ones or when the skin can't load. */
export default function Avatar({ size = 'h-9 w-9' }: { size?: string }) {
    const { config } = useConfig();
    const [failed, setFailed] = useState(false);
    const hasSkin = Boolean(config.minecraft_token) && Boolean(config.uuid);

    useEffect(() => setFailed(false), [config.uuid]);

    if (hasSkin && !failed) {
        return (
            <img
                src={`https://crafatar.com/avatars/${encodeURIComponent(config.uuid ?? "")}?size=64&overlay`}
                alt=''
                onError={() => setFailed(true)}
                className={`${size} shrink-0 rounded-lg bg-black/30 [image-rendering:pixelated]`}
            />
        );
    }

    return (
        <div className={`${size} flex shrink-0 items-center justify-center rounded-lg bg-accent/15 font-semibold uppercase text-accent`}>
            {config.username.slice(0, 1)}
        </div>
    );
}
