'use client';
import Link from 'next/link';
import Image from 'next/image';
import logoUrl from '@/images/rivet-logos/icon-text-white.svg';
import { Header as RivetHeader } from '@rivet-gg/components/header';
import { Button, cn } from '@rivet-gg/components';
import { ReactNode, useState } from 'react';
import { DocsMobileNavigation } from '@/components/DocsMobileNavigation';
import { faDiscord, faGithub, Icon } from '@rivet-gg/icons';
import { AnimatePresence, motion } from 'unframer';

interface HeaderProps {
  active?: 'product' | 'docs' | 'blog' | 'pricing';
  subnav?: ReactNode;
  mobileBreadcrumbs?: ReactNode;
}

export function Header({ active, subnav }: HeaderProps) {
  const [isSubnavOpen, setIsSubnavOpen] = useState(false);

  const headerStyles = cn({
    'md:border-transparent md:static md:bg-transparent md:rounded-2xl md:max-w-[1200px] md:border-transparent md:backdrop-none [&>div:first-child]:px-3':
      !active
  });
  return (
    <>
      <div
        className={cn('fixed inset-0 z-10 backdrop-blur-sm', isSubnavOpen ? 'opacity-100' : 'opacity-0')}
      />
      <motion.div
        className='z-10 md:fixed md:left-1/2 md:top-4 md:w-full md:-translate-x-1/2 md:px-8'
        onMouseLeave={() => setIsSubnavOpen(false)}>
        <motion.div className='relative  before:pointer-events-none before:absolute before:inset-0 before:z-20 before:rounded-2xl before:border before:border-white/10 before:content-[""]'>
          <motion.div
            className={cn(
              'absolute inset-0 rounded-2xl backdrop-blur-md backdrop-saturate-[140%] transition-colors',
              isSubnavOpen ? 'bg-background' : 'bg-white/5 bg-gradient-to-r from-white/5 to-black/10'
            )}
          />
          <RivetHeader
            className={headerStyles}
            logo={
              <Link href='/'>
                <Image {...logoUrl} className='ml-1 w-20' alt='Rivet logo' />
              </Link>
            }
            subnav={subnav}
            support={
              <div className='flex flex-col gap-4 font-v2 subpixel-antialiased'>
                <RivetHeader.NavItem asChild>
                  <Link href='https://hub.rivet.gg'>Sign In</Link>
                </RivetHeader.NavItem>
                <RivetHeader.NavItem asChild>
                  <Link href='/discord'>Discord</Link>
                </RivetHeader.NavItem>
                <RivetHeader.NavItem asChild>
                  <Link href='/support'>Support</Link>
                </RivetHeader.NavItem>
              </div>
            }
            links={
              <>
                <RivetHeader.NavItem asChild className='-m-2 p-2'>
                  <Link href='/discord' className='text-white/90'>
                    <Icon icon={faDiscord} className='drop-shadow-md' />
                  </Link>
                </RivetHeader.NavItem>
                <RivetHeader.NavItem asChild className='p-2'>
                  <Link href='/support' className='text-white/90'>
                    <Icon icon={faGithub} className='drop-shadow-md' />
                  </Link>
                </RivetHeader.NavItem>
                <Button variant='secondary' asChild className='font-v2 subpixel-antialiased'>
                  <Link href='https://hub.rivet.gg'>Sign In</Link>
                </Button>
              </>
            }
            mobileBreadcrumbs={<DocsMobileNavigation />}
            breadcrumbs={
              <div className='flex items-center gap-5 font-v2 subpixel-antialiased'>
                <RivetHeader.NavItem
                  asChild
                  className='flex items-center gap-1 py-2 '
                  onMouseEnter={() => setIsSubnavOpen(true)}>
                  <Link
                    href='/docs'
                    className='text-white/90'
                    aria-current={active === 'product' ? 'page' : undefined}>
                    Product
                  </Link>
                </RivetHeader.NavItem>
                <RivetHeader.NavItem asChild className='flex items-center gap-1 py-2'>
                  <Link
                    href='/docs'
                    className='text-white/90'
                    aria-current={active === 'docs' ? 'page' : undefined}>
                    Docs
                  </Link>
                </RivetHeader.NavItem>
                <RivetHeader.NavItem asChild className='flex items-center gap-1'>
                  <Link
                    href='/changelog'
                    className='text-white/90'
                    aria-current={active === 'blog' ? 'page' : undefined}>
                    Changelog
                  </Link>
                </RivetHeader.NavItem>
                <RivetHeader.NavItem asChild className='flex items-center gap-1'>
                  <Link
                    href='/pricing'
                    className='text-white/90'
                    aria-current={active === 'pricing' ? 'page' : undefined}>
                    Pricing
                  </Link>
                </RivetHeader.NavItem>
              </div>
            }
          />
          {/* <motion.div className='overflow-hidden'> */}
          <AnimatePresence>
            {isSubnavOpen ? (
              <motion.div
                initial={{ height: 0 }}
                animate={{ opacity: 1, height: 270 }}
                exit={{ opacity: 0, height: 0 }}
                className='relative  overflow-hidden'>
                <div className='grid grid-cols-6 grid-rows-3 gap-4 px-4 pb-2'>
                  <HeaderSubItem className='col-span-2 row-span-3 h-[calc(270px-1rem)]'>
                    <div className='relative z-10 h-full'>
                      <p className='font-bold opacity-80 transition-opacity group-hover:opacity-100'>
                        Feature A
                      </p>
                      <p className='opacity-80 transition-opacity group-hover:opacity-100'>
                        Description of Feature A, this can be long and descriptive. Hover over to see an
                        effect.
                      </p>
                      <Button variant='secondary' size='sm' asChild className='absolute bottom-0 right-0 '>
                        <div>Read more</div>
                      </Button>
                    </div>
                    <Image
                      src='https://picsum.photos/seed/test/600/600'
                      fill
                      alt='Image'
                      className='object-cover opacity-40'
                    />
                  </HeaderSubItem>
                  <div className='col-span-2 row-span-3 rounded-lg border border-white/10 bg-white/5 p-4' />
                  <div className='col-span-2 rounded-lg border border-white/10 bg-white/5 p-4' />
                  <div className='col-span-2 rounded-lg border border-white/10 bg-white/5 p-4' />
                  <div className='col-span-2 rounded-lg border border-white/10 bg-white/5 p-4' />
                </div>
              </motion.div>
            ) : null}
          </AnimatePresence>
          {/* </motion.div> */}
        </motion.div>
      </motion.div>
    </>
  );
}

interface HeaderSubItemProps {
  className?: string;
  children?: ReactNode;
}
function HeaderSubItem({ className, children }: HeaderSubItemProps) {
  return (
    <div
      className={cn(
        'group overflow-hidden rounded-md p-4 text-sm grayscale transition-all hover:grayscale-0',
        className
      )}>
      {children}
    </div>
  );
}
