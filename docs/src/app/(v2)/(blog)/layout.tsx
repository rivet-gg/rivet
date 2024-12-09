import { Header } from '@/components/v2/Header';

export default function Layout({ children }) {
  return (
    <>
      <Header active='blog' />
      <div className='mx-auto mt-20 w-full max-w-6xl px-8 md:mt-32'>{children}</div>
    </>
  );
}
