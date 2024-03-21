import logo from '@/assets/HyClyint.png';

type Props = {
    loading: boolean;
};

export default function Loading({ loading }: Props) {
    return (
        <div
            className={`absolute top-0 left-0 text-white bg-hyclyint w-full h-screen flex items-center justify-center flex-col transition-all duration-500 ease-in-out ${
                loading ? 'opacity-100' : 'opacity-0'
            }`}
        >
            <img src={logo} alt='HyClyint' className='w-1/4' />
        </div>
    );
}
