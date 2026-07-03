'use client';

import { useState, useEffect } from 'react';
import FluxVisualizer from '@/components/FluxVisualizer';

interface BatchInfo {
  id: number;
  status: 'pending' | 'processing' | 'settled' | 'failed';
  amount: bigint;
  timestamp: number;
  participants: number;
}

interface AccountInfo {
  address: string;
  balance: bigint;
  lastUpdated: number;
}

export default function Dashboard() {
  const [batches, setBatches] = useState<BatchInfo[]>([]);
  const [accounts, setAccounts] = useState<AccountInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const loadDashboardData = async () => {
      try {
        setLoading(true);
        setError(null);

        // Simulate loading batch data
        const mockBatches: BatchInfo[] = [
          {
            id: 1,
            status: 'settled',
            amount: 1000000n,
            timestamp: Date.now() - 300000,
            participants: 5,
          },
          {
            id: 2,
            status: 'processing',
            amount: 500000n,
            timestamp: Date.now() - 60000,
            participants: 3,
          },
          {
            id: 3,
            status: 'pending',
            amount: 750000n,
            timestamp: Date.now(),
            participants: 2,
          },
        ];

        // Simulate loading account data
        const mockAccounts: AccountInfo[] = [
          {
            address: 'GXXXX...XXXX',
            balance: 5000000n,
            lastUpdated: Date.now(),
          },
          {
            address: 'GYYYY...YYYY',
            balance: 3500000n,
            lastUpdated: Date.now(),
          },
          {
            address: 'GZZZZ...ZZZZ',
            balance: 2100000n,
            lastUpdated: Date.now(),
          },
        ];

        setBatches(mockBatches);
        setAccounts(mockAccounts);
      } catch (err) {
        setError(
          err instanceof Error ? err.message : 'Failed to load dashboard data'
        );
      } finally {
        setLoading(false);
      }
    };

    loadDashboardData();

    // Poll for updates every 30 seconds
    const interval = setInterval(loadDashboardData, 30000);
    return () => clearInterval(interval);
  }, []);

  const formatAmount = (amount: bigint): string => {
    const fixed = Number(amount) / 10000000; // Convert from fixed-point
    return fixed.toLocaleString('en-US', {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    });
  };

  const formatTimestamp = (timestamp: number): string => {
    return new Date(timestamp).toLocaleString();
  };

  const getStatusColor = (
    status: BatchInfo['status']
  ): 'bg-green-900' | 'bg-blue-900' | 'bg-yellow-900' | 'bg-red-900' => {
    switch (status) {
      case 'settled':
        return 'bg-green-900';
      case 'processing':
        return 'bg-blue-900';
      case 'pending':
        return 'bg-yellow-900';
      case 'failed':
        return 'bg-red-900';
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-slate-200">Loading dashboard...</div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-slate-950 p-8">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-12">
          <h1 className="text-4xl font-bold text-slate-50 mb-2">
            Soroban Flux Dashboard
          </h1>
          <p className="text-slate-400">
            Real-time distributed flux token management
          </p>
        </div>

        {/* Error Display */}
        {error && (
          <div className="mb-8 p-4 bg-red-900/20 border border-red-700 rounded-lg">
            <p className="text-red-200">{error}</p>
          </div>
        )}

        {/* Flux Visualizer */}
        <div className="mb-12">
          <FluxVisualizer batches={batches} accounts={accounts} />
        </div>

        {/* Statistics Grid */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          <div className="bg-slate-900 border border-slate-700 rounded-lg p-6">
            <h3 className="text-slate-400 text-sm font-medium mb-2">
              Active Batches
            </h3>
            <p className="text-3xl font-bold text-slate-50">
              {batches.filter((b) => b.status === 'processing').length}
            </p>
          </div>

          <div className="bg-slate-900 border border-slate-700 rounded-lg p-6">
            <h3 className="text-slate-400 text-sm font-medium mb-2">
              Settled Batches
            </h3>
            <p className="text-3xl font-bold text-slate-50">
              {batches.filter((b) => b.status === 'settled').length}
            </p>
          </div>

          <div className="bg-slate-900 border border-slate-700 rounded-lg p-6">
            <h3 className="text-slate-400 text-sm font-medium mb-2">
              Managed Accounts
            </h3>
            <p className="text-3xl font-bold text-slate-50">{accounts.length}</p>
          </div>
        </div>

        {/* Settlement Batches Table */}
        <div className="mb-12">
          <h2 className="text-2xl font-bold text-slate-50 mb-6">
            Settlement Batches
          </h2>
          <div className="overflow-x-auto bg-slate-900 border border-slate-700 rounded-lg">
            <table className="w-full">
              <thead>
                <tr className="border-b border-slate-700 bg-slate-800">
                  <th className="px-6 py-4 text-left text-sm font-medium text-slate-300">
                    Batch ID
                  </th>
                  <th className="px-6 py-4 text-left text-sm font-medium text-slate-300">
                    Status
                  </th>
                  <th className="px-6 py-4 text-left text-sm font-medium text-slate-300">
                    Amount
                  </th>
                  <th className="px-6 py-4 text-left text-sm font-medium text-slate-300">
                    Participants
                  </th>
                  <th className="px-6 py-4 text-left text-sm font-medium text-slate-300">
                    Created
                  </th>
                </tr>
              </thead>
              <tbody>
                {batches.map((batch) => (
                  <tr
                    key={batch.id}
                    className="border-b border-slate-700 hover:bg-slate-800"
                  >
                    <td className="px-6 py-4 text-sm text-slate-50">
                      #{batch.id}
                    </td>
                    <td className="px-6 py-4 text-sm">
                      <span
                        className={`px-3 py-1 rounded-full text-xs font-medium ${getStatusColor(batch.status)} text-slate-50`}
                      >
                        {batch.status}
                      </span>
                    </td>
                    <td className="px-6 py-4 text-sm text-slate-50">
                      {formatAmount(batch.amount)}
                    </td>
                    <td className="px-6 py-4 text-sm text-slate-50">
                      {batch.participants}
                    </td>
                    <td className="px-6 py-4 text-sm text-slate-400">
                      {formatTimestamp(batch.timestamp)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        {/* Accounts Table */}
        <div>
          <h2 className="text-2xl font-bold text-slate-50 mb-6">Accounts</h2>
          <div className="overflow-x-auto bg-slate-900 border border-slate-700 rounded-lg">
            <table className="w-full">
              <thead>
                <tr className="border-b border-slate-700 bg-slate-800">
                  <th className="px-6 py-4 text-left text-sm font-medium text-slate-300">
                    Address
                  </th>
                  <th className="px-6 py-4 text-left text-sm font-medium text-slate-300">
                    Balance
                  </th>
                  <th className="px-6 py-4 text-left text-sm font-medium text-slate-300">
                    Last Updated
                  </th>
                </tr>
              </thead>
              <tbody>
                {accounts.map((account) => (
                  <tr
                    key={account.address}
                    className="border-b border-slate-700 hover:bg-slate-800"
                  >
                    <td className="px-6 py-4 text-sm text-slate-50 font-mono">
                      {account.address}
                    </td>
                    <td className="px-6 py-4 text-sm text-slate-50">
                      {formatAmount(account.balance)}
                    </td>
                    <td className="px-6 py-4 text-sm text-slate-400">
                      {formatTimestamp(account.lastUpdated)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
  );
}
