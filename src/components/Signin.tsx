import { useForm } from 'react-hook-form';
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from './ui/form';
import { Input } from './ui/input';
import * as z from 'zod';
import { zodResolver } from '@hookform/resolvers/zod';
import { Button } from './ui/button';
import { Loader2 } from 'lucide-react';
import { tauri } from '@tauri-apps/api';
import { useEffect, useState } from 'react';
import { Dialog, DialogContent } from './ui/dialog';
import { listen } from '@tauri-apps/api/event';

const schema = z.object({
    username: z.string().min(3).max(32),
});

type Props = {};

export default function Signin({}: Props) {
    const [disabled, setDisabled] = useState(false);
    const [dialogOpen, setDialogOpen] = useState(false);
    const [code, setCode] = useState('');
    const [link, setLink] = useState('');
    const form = useForm({
        resolver: zodResolver(schema),
        defaultValues: {
            username: 'Steve',
        },
    });

    async function onSubmit(data: z.infer<typeof schema>) {
        await tauri.invoke('login_offline', { username: data.username });
    }

    async function onGenerateCode() {
        setDisabled(true);
        tauri.invoke('login_microsoft');
    }

    useEffect(() => {
        const cancel = listen<String>('microsoft-code', (event) => {
            const [link, code] = event.payload.split(' ');
            setLink(link);
            setCode(code);
            setDialogOpen(true);
            setDisabled(false);
        });

        return () => {
            cancel.then((cancel) => cancel());
        }
    }, []);

    return (
        <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
            <div className='w-full h-full p-8'>
                <div className='w-full h-full bg-first opacity-75 rounded-lg text-white flex flex-col items-center justify-center gap-20'>
                    <h1 className='scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl'>SIGN IN</h1>
                    <Form {...form}>
                        <form onSubmit={form.handleSubmit(onSubmit)} className='space-y-2 w-1/2'>
                            <FormField
                                name='username'
                                render={({ field }) => (
                                    <FormItem>
                                        <FormLabel>Username</FormLabel>
                                        <FormControl>
                                            <Input {...field} />
                                        </FormControl>
                                        <FormMessage />
                                    </FormItem>
                                )}
                            />
                            <div className='flex flex-row gap-4 w-full'>
                                <Button type='submit' variant='secondary' className='flex-1'>
                                    Sign in offline
                                </Button>
                                <Button type='button' onClick={onGenerateCode} disabled={disabled} variant='secondary' className='flex-1'>
                                    {disabled ? (
                                        <>
                                            <Loader2 className='w-4 h-4 animate-spin mr-2' />
                                            Loading Code
                                        </>
                                    ) : (
                                        <>Sign in with Microsoft</>
                                    )}
                                </Button>
                            </div>
                        </form>
                    </Form>
                </div>
            </div>
            <DialogContent className='bg-first opacity-75'>
                <div className='w-full h-full rounded-lg text-white flex flex-col items-center justify-center gap-20'>
                    <p className='text-lg'>
                        Please sign in with Microsoft by clicking the button below. A new tab will open with the Microsoft sign in page. After signing in, you
                        will be given a code. Enter the code in the input field and click the "Sign in" button.
                    </p>
                    <a href={link} target='_blank' rel='noreferrer' className='text-lg underline'>
                        Sign in with Microsoft
                    </a>
                    <p className='text-lg'>Your code is: {code}</p>
                </div>
            </DialogContent>
        </Dialog>
    );
}
