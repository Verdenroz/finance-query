import type { MarketIndex } from '../services/api';

interface MarketOverviewProps {
  indices: MarketIndex[];
}

const MarketOverview = ({ indices }: MarketOverviewProps) => {
  return (
    <div className="bg-white rounded-lg shadow p-6">
      <h2 className="text-xl font-bold text-gray-900 mb-4">Market Indices</h2>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {indices.map((index) => (
          <div key={index.index} className="border border-gray-200 rounded-lg p-4">
            <div className="text-sm text-gray-600 mb-1">{index.name}</div>
            <div className="text-2xl font-bold text-gray-900">{index.price}</div>
            <div className="flex items-center space-x-2 mt-1">
              <span className="text-sm font-medium text-gray-700">{index.change}</span>
              <span
                className={`text-sm font-semibold ${
                  index.percentChange.startsWith('+') ? 'text-green-600' : 'text-red-600'
                }`}
              >
                {index.percentChange}
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default MarketOverview;
