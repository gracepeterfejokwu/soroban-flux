'use client';

import React, { useMemo } from 'react';

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

interface FluxVisualizerProps {
  batches: BatchInfo[];
  accounts: AccountInfo[];
}

export default function FluxVisualizer({
  batches,
  accounts,
}: FluxVisualizerProps) {
  const stats = useMemo(() => {
    const totalAmount = batches.reduce((sum, b) => sum + b.amount, 0n);
    const totalBalance = accounts.reduce((sum, a) => sum + a.balance, 0n);
    const avgBatchSize =
      batches.length > 0 ? totalAmount / BigInt(batches.length) : 0n;

    return {
      totalAmount,
      totalBalance,
      avgBatchSize,
      batchCount: batches.length,
      accountCount: accounts.length,
      settledCount: batches.filter((b) => b.status === 'settled').length,
      processingCount: batches.filter((b) => b.status === 'processing').length,
      failedCount: batches.filter((b) => b.status === 'failed').length,
    };
  }, [batches, accounts]);

  const formatAmount = (amount: bigint): string => {
    const fixed = Number(amount) / 10000000; // Convert from fixed-point
    return fixed.toLocaleString('en-US', {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    });
  };

  const calculateFlowRate = (batch: BatchInfo): string => {
    if (batch.participants === 0) return '0.00';
    const perParticipant = Number(batch.amount) / batch.participants;
    return (perParticipant / 10000000).toFixed(2);
  };

  const getStatusIcon = (
    status: BatchInfo['status']
  ): React.ReactNode => {
    switch (status) {
      case 'settled':
        return <CheckIcon />;
      case 'processing':
        return <SpinnerIcon />;
      case 'pending':
        return <PendingIcon />;
      case 'failed':
        return <ErrorIcon />;
    }
  };

  const renderFlowDiagram = (): React.ReactNode => {
    if (batches.length === 0) {
      return (
        <div className="text-center text-slate-400 py-8">
          No active batches to visualize
        </div>
      );
    }

    return (
      <div className="space-y-4">
        {batches.map((batch) => {
          const width = Math.min(
            (Number(batch.amount) / Number(stats.totalAmount)) * 100,
            100
          );

          return (
            <div key={batch.id} className="space-y-2">
              <div className="flex justify-between items-center">
                <div className="flex items-center gap-2">
                  {getStatusIcon(batch.status)}
                  <span className="text-sm font-medium text-slate-50">
                    Batch #{batch.id}
                  </span>
                </div>
                <span className="text-xs text-slate-400">
                  {formatAmount(batch.amount)}
                </span>
              </div>
              <div className="w-full bg-slate-800 rounded h-2 overflow-hidden">
                <div
                  className={`h-full transition-all ${getBarColor(batch.status)}`}
                  style={{ width: `${width || 2}%` }}
                />
              </div>
              <div className="flex justify-between text-xs text-slate-400">
                <span>{batch.participants} participants</span>
                <span>per participant: {calculateFlowRate(batch)}</span>
              </div>
            </div>
          );
        })}
      </div>
    );
  };

  return (
    <div className="space-y-8">
      {/* Key Metrics */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <div className="bg-slate-900 border border-slate-700 rounded-lg p-4">
          <p className="text-xs text-slate-400 mb-2">Total Volume</p>
          <p className="text-2xl font-bold text-slate-50">
            {formatAmount(stats.totalAmount)}
          </p>
        </div>
        <div className="bg-slate-900 border border-slate-700 rounded-lg p-4">
          <p className="text-xs text-slate-400 mb-2">Avg Batch Size</p>
          <p className="text-2xl font-bold text-slate-50">
            {formatAmount(stats.avgBatchSize)}
          </p>
        </div>
        <div className="bg-slate-900 border border-slate-700 rounded-lg p-4">
          <p className="text-xs text-slate-400 mb-2">Settled ✓</p>
          <p className="text-2xl font-bold text-green-400">
            {stats.settledCount}
          </p>
        </div>
        <div className="bg-slate-900 border border-slate-700 rounded-lg p-4">
          <p className="text-xs text-slate-400 mb-2">Processing ⟳</p>
          <p className="text-2xl font-bold text-blue-400">
            {stats.processingCount}
          </p>
        </div>
      </div>

      {/* Flow Visualization */}
      <div className="bg-slate-900 border border-slate-700 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-slate-50 mb-6">
          Settlement Flow
        </h3>
        {renderFlowDiagram()}
      </div>

      {/* Network Distribution */}
      <div className="bg-slate-900 border border-slate-700 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-slate-50 mb-6">
          Account Distribution
        </h3>
        {accounts.length > 0 ? (
          <div className="space-y-3">
            {accounts.map((account) => {
              const percentage = (Number(account.balance) / Number(stats.totalBalance)) * 100;
              return (
                <div key={account.address}>
                  <div className="flex justify-between items-center mb-1">
                    <span className="text-sm text-slate-300 font-mono">
                      {account.address}
                    </span>
                    <span className="text-sm font-medium text-slate-50">
                      {percentage.toFixed(1)}%
                    </span>
                  </div>
                  <div className="w-full bg-slate-800 rounded h-2">
                    <div
                      className="h-full bg-gradient-to-r from-blue-500 to-purple-500 rounded transition-all"
                      style={{ width: `${percentage}%` }}
                    />
                  </div>
                </div>
              );
            })}
          </div>
        ) : (
          <div className="text-center text-slate-400 py-8">
            No account data available
          </div>
        )}
      </div>
    </div>
  );
}

// Icons
function CheckIcon() {
  return (
    <svg
      className="w-4 h-4 text-green-400"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M5 13l4 4L19 7"
      />
    </svg>
  );
}

function SpinnerIcon() {
  return (
    <svg
      className="w-4 h-4 text-blue-400 animate-spin"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
      />
    </svg>
  );
}

function PendingIcon() {
  return (
    <svg
      className="w-4 h-4 text-yellow-400"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
      />
    </svg>
  );
}

function ErrorIcon() {
  return (
    <svg
      className="w-4 h-4 text-red-400"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M12 8v4m0 4v.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
      />
    </svg>
  );
}

function getBarColor(status: BatchInfo['status']): string {
  switch (status) {
    case 'settled':
      return 'bg-green-500';
    case 'processing':
      return 'bg-blue-500';
    case 'pending':
      return 'bg-yellow-500';
    case 'failed':
      return 'bg-red-500';
  }
}
