import type { Metadata } from 'next';
import './globals.css';

export const metadata: Metadata = {
  title: 'Soroban Flux Dashboard',
  description: 'Real-time distributed flux token management',
  viewport: {
    width: 'device-width',
    initialScale: 1,
    maximumScale: 1,
  },
  robots: 'noindex, nofollow',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <head>
        <meta charSet="utf-8" />
        <link rel="icon" href="/favicon.ico" />
      </head>
      <body>
        <div id="root" className="min-h-screen bg-slate-950">
          {children}
        </div>
      </body>
    </html>
  );
}
