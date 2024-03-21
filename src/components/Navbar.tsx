type Props = {
    setSelected: (selected: string) => void;
};

export default function Navbar({ setSelected }: Props) {
    return (
        <div className='w-full text-white h-[10%] bg-first opacity-75 gap-3 rounded-b-md flex items-center justify-center'>
            <button
                onClick={() => setSelected('news')}
                className='bg-second font-extrabold text-xl p-4 px-8 rounded-md hover:text-fifth transition-colors duration-150'
            >
                NEWS
            </button>
            <button
                onClick={() => setSelected('settings')}
                className='bg-second font-extrabold text-xl p-4 px-8 rounded-md hover:text-fifth transition-colors duration-150'
            >
                SETTINGS
            </button>
            <button
                onClick={() => setSelected('mods')}
                className='bg-second font-extrabold text-xl p-4 px-8 rounded-md hover:text-fifth transition-colors duration-150'
            >
                MODS
            </button>
            <a
                className='bg-second font-extrabold text-xl p-4 px-8 rounded-md hover:text-fifth transition-colors duration-150'
                href='https://discord.gg/99zb4wvxzs'
                target='_blank'
            >
                DISCORD
            </a>
        </div>
    );
}
