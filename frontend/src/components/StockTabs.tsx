import { useState } from 'react';
import type { Quote, News, Holders, Financials } from '../services/api';
import NewsSection from './NewsSection';

interface StockTabsProps {
  quote: Quote;
  news: News[];
  holders: Holders | null;
  financials: Financials | null;
}

const StockTabs = ({ quote, news, holders, financials }: StockTabsProps) => {
  const [activeTab, setActiveTab] = useState<'overview' | 'news' | 'financials' | 'holders'>('overview');

  const tabs = [
    { id: 'overview' as const, label: 'Overview' },
    { id: 'news' as const, label: 'News' },
    { id: 'financials' as const, label: 'Financials' },
    { id: 'holders' as const, label: 'Holders' },
  ];

  return (
    <div className="bg-white rounded-lg shadow">
      <div className="border-b border-gray-200">
        <nav className="flex space-x-8 px-6">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`py-4 px-1 border-b-2 font-medium text-sm transition ${
                activeTab === tab.id
                  ? 'border-blue-600 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </nav>
      </div>

      <div className="p-6">
        {activeTab === 'overview' && (
          <div className="space-y-6">
            <div>
              <h3 className="text-lg font-semibold text-gray-900 mb-3">About</h3>
              <p className="text-gray-700 leading-relaxed">{quote.about || 'No description available.'}</p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <h3 className="text-lg font-semibold text-gray-900 mb-3">Key Statistics</h3>
                <div className="space-y-3">
                  {quote.yearHigh && (
                    <div className="flex justify-between">
                      <span className="text-gray-600">52 Week High</span>
                      <span className="font-semibold">${quote.yearHigh}</span>
                    </div>
                  )}
                  {quote.yearLow && (
                    <div className="flex justify-between">
                      <span className="text-gray-600">52 Week Low</span>
                      <span className="font-semibold">${quote.yearLow}</span>
                    </div>
                  )}
                  {quote.avgVolume && (
                    <div className="flex justify-between">
                      <span className="text-gray-600">Avg Volume</span>
                      <span className="font-semibold">{quote.avgVolume.toLocaleString()}</span>
                    </div>
                  )}
                  {quote.dividend && (
                    <div className="flex justify-between">
                      <span className="text-gray-600">Dividend</span>
                      <span className="font-semibold">${quote.dividend}</span>
                    </div>
                  )}
                  {quote.yield && (
                    <div className="flex justify-between">
                      <span className="text-gray-600">Yield</span>
                      <span className="font-semibold">{quote.yield}</span>
                    </div>
                  )}
                  {quote.earningsDate && (
                    <div className="flex justify-between">
                      <span className="text-gray-600">Earnings Date</span>
                      <span className="font-semibold">{quote.earningsDate}</span>
                    </div>
                  )}
                </div>
              </div>

              <div>
                <h3 className="text-lg font-semibold text-gray-900 mb-3">Performance</h3>
                <div className="space-y-3">
                  {quote.fiveDaysReturn && (
                    <div className="flex justify-between">
                      <span className="text-gray-600">5 Days</span>
                      <span className={`font-semibold ${quote.fiveDaysReturn.startsWith('+') ? 'text-green-600' : 'text-red-600'}`}>
                        {quote.fiveDaysReturn}
                      </span>
                    </div>
                  )}
                  {quote.oneMonthReturn && (
                    <div className="flex justify-between">
                      <span className="text-gray-600">1 Month</span>
                      <span className={`font-semibold ${quote.oneMonthReturn.startsWith('+') ? 'text-green-600' : 'text-red-600'}`}>
                        {quote.oneMonthReturn}
                      </span>
                    </div>
                  )}
                  {quote.ytdReturn && (
                    <div className="flex justify-between">
                      <span className="text-gray-600">YTD</span>
                      <span className={`font-semibold ${quote.ytdReturn.startsWith('+') ? 'text-green-600' : 'text-red-600'}`}>
                        {quote.ytdReturn}
                      </span>
                    </div>
                  )}
                  {quote.yearReturn && (
                    <div className="flex justify-between">
                      <span className="text-gray-600">1 Year</span>
                      <span className={`font-semibold ${quote.yearReturn.startsWith('+') ? 'text-green-600' : 'text-red-600'}`}>
                        {quote.yearReturn}
                      </span>
                    </div>
                  )}
                  {quote.threeYearReturn && (
                    <div className="flex justify-between">
                      <span className="text-gray-600">3 Years</span>
                      <span className={`font-semibold ${quote.threeYearReturn.startsWith('+') ? 'text-green-600' : 'text-red-600'}`}>
                        {quote.threeYearReturn}
                      </span>
                    </div>
                  )}
                  {quote.fiveYearReturn && (
                    <div className="flex justify-between">
                      <span className="text-gray-600">5 Years</span>
                      <span className={`font-semibold ${quote.fiveYearReturn.startsWith('+') ? 'text-green-600' : 'text-red-600'}`}>
                        {quote.fiveYearReturn}
                      </span>
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'news' && (
          <NewsSection news={news} title={`${quote.symbol} News`} />
        )}

        {activeTab === 'financials' && (
          <div>
            {financials ? (
              <div className="space-y-4">
                <h3 className="text-lg font-semibold text-gray-900">Financial Data</h3>
                <p className="text-gray-600">
                  Financial statements and detailed metrics are available through the API.
                </p>
                <pre className="bg-gray-50 p-4 rounded overflow-auto text-sm">
                  {JSON.stringify(financials, null, 2)}
                </pre>
              </div>
            ) : (
              <p className="text-gray-600">No financial data available.</p>
            )}
          </div>
        )}

        {activeTab === 'holders' && (
          <div>
            {holders ? (
              <div className="space-y-6">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  {holders.institutionalOwnership !== undefined && (
                    <div className="border border-gray-200 rounded-lg p-4">
                      <div className="text-sm text-gray-600">Institutional Ownership</div>
                      <div className="text-2xl font-bold text-gray-900 mt-1">
                        {(holders.institutionalOwnership * 100).toFixed(2)}%
                      </div>
                    </div>
                  )}
                  {holders.insiderOwnership !== undefined && (
                    <div className="border border-gray-200 rounded-lg p-4">
                      <div className="text-sm text-gray-600">Insider Ownership</div>
                      <div className="text-2xl font-bold text-gray-900 mt-1">
                        {(holders.insiderOwnership * 100).toFixed(2)}%
                      </div>
                    </div>
                  )}
                </div>

                {holders.majorHolders && holders.majorHolders.length > 0 && (
                  <div>
                    <h3 className="text-lg font-semibold text-gray-900 mb-3">Major Holders</h3>
                    <div className="overflow-x-auto">
                      <table className="min-w-full divide-y divide-gray-200">
                        <thead className="bg-gray-50">
                          <tr>
                            <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                              Name
                            </th>
                            <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                              Shares
                            </th>
                            <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                              Value
                            </th>
                            <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                              % Held
                            </th>
                          </tr>
                        </thead>
                        <tbody className="bg-white divide-y divide-gray-200">
                          {holders.majorHolders.map((holder, index) => (
                            <tr key={index}>
                              <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                                {holder.name}
                              </td>
                              <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                {holder.shares.toLocaleString()}
                              </td>
                              <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                ${holder.value.toLocaleString()}
                              </td>
                              <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                {holder.percentHeld.toFixed(2)}%
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  </div>
                )}
              </div>
            ) : (
              <p className="text-gray-600">No holder data available.</p>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

export default StockTabs;
