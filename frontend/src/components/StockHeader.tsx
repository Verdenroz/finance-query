import type { Quote } from '../services/api';

interface StockHeaderProps {
  quote: Quote;
}

const StockHeader = ({ quote }: StockHeaderProps) => {
  const isPositive = quote.change.startsWith('+') || !quote.change.startsWith('-');

  return (
    <div className="bg-white rounded-lg shadow p-6">
      <div className="flex items-start justify-between">
        <div className="flex items-center space-x-4">
          {quote.logo && (
            <img src={quote.logo} alt={quote.name} className="w-16 h-16 rounded" />
          )}
          <div>
            <h1 className="text-3xl font-bold text-gray-900">{quote.symbol}</h1>
            <p className="text-lg text-gray-600">{quote.name}</p>
            {quote.sector && (
              <p className="text-sm text-gray-500">
                {quote.sector} {quote.industry && `• ${quote.industry}`}
              </p>
            )}
          </div>
        </div>
        <div className="text-right">
          <div className="text-3xl font-bold text-gray-900">${quote.price}</div>
          <div className={`text-lg font-semibold ${isPositive ? 'text-green-600' : 'text-red-600'}`}>
            {quote.change} ({quote.percentChange})
          </div>
          {quote.afterHoursPrice && (
            <div className="text-sm text-gray-600 mt-1">
              After Hours: ${quote.afterHoursPrice}
            </div>
          )}
        </div>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4 mt-6 pt-6 border-t border-gray-200">
        {quote.open && (
          <div>
            <div className="text-sm text-gray-600">Open</div>
            <div className="font-semibold text-gray-900">${quote.open}</div>
          </div>
        )}
        {quote.high && (
          <div>
            <div className="text-sm text-gray-600">High</div>
            <div className="font-semibold text-gray-900">${quote.high}</div>
          </div>
        )}
        {quote.low && (
          <div>
            <div className="text-sm text-gray-600">Low</div>
            <div className="font-semibold text-gray-900">${quote.low}</div>
          </div>
        )}
        {quote.marketCap && (
          <div>
            <div className="text-sm text-gray-600">Market Cap</div>
            <div className="font-semibold text-gray-900">{quote.marketCap}</div>
          </div>
        )}
        {quote.pe && (
          <div>
            <div className="text-sm text-gray-600">P/E Ratio</div>
            <div className="font-semibold text-gray-900">{quote.pe}</div>
          </div>
        )}
        {quote.volume && (
          <div>
            <div className="text-sm text-gray-600">Volume</div>
            <div className="font-semibold text-gray-900">{quote.volume.toLocaleString()}</div>
          </div>
        )}
      </div>
    </div>
  );
};

export default StockHeader;
