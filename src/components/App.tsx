import { useState } from 'react';
import { useConfig } from '../context/ConfigContext';
import Footer from './Footer';
import Navbar from './Navbar';
import NewsComponent from './NewsComponent';
import Signin from './Signin';
import SettingsComponent from './SettingsComponent';

type Props = {};

export default function App({}: Props) {
    const [selected, setSelected] = useState('news');
    const { config } = useConfig();

    return (
        <div className='bg-minecraft dark bg-cover w-full h-screen overflow-hidden'>
            {config.is_logged_in ? (
                <>
                    <Navbar setSelected={setSelected} />
                    <div className='w-full h-[80%] p-10'>
                        {selected === 'news' && <NewsComponent />}
                        {selected === 'settings' && <SettingsComponent />}
                    </div>
                    <Footer />
                </>
            ) : (
                <Signin />
            )}
        </div>
    );
}
