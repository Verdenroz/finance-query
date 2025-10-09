import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { TrendingUp, TrendingDown, Activity } from 'lucide-react';
import { getIndices, getMarketMovers, getSectors, getNews, searchSymbols, type MarketIndex, type MarketMover, type MarketSector, type News } from '../services/api';
import MarketOverview from '../components/MarketOverview';
import NewsSection from '../components/NewsSection';
import SearchBar from '../components/SearchBar';

const HomePage = () => {
  const navigate = useNavigate();
  const [indices, setIndices] = useState<MarketIndex[]>([]);
  const [gainers, setGainers] = useState<MarketMover[]>([]);
  const [losers, setLosers] = useState<MarketMover[]>([]);
  const [actives, setActives] = useState<MarketMover[]>([]);
  const [sectors, setSectors] = useState<MarketSector[]>([]);
  const [news, setNews] = useState<News[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [indicesData, gainersData, losersData, activesData, sectorsData, newsData] = await Promise.all([
          getIndices(),
          getMarketMovers('gainers', 10),
          getMarketMovers('losers', 10),
          getMarketMovers('actives', 10),
          getSectors(),
          getNews(),
        ]);

        setIndices(indicesData);
        setGainers(gainersData);
        setLosers(losersData);
        setActives(activesData);
        setSectors(sectorsData);
        setNews(newsData);
      } catch (error) {
        console.error('Error fetching market data:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, []);

  const handleSearch = async (query: string) => {
    try {
      const results = await searchSymbols(query);
      if (results.length > 0) {
        navigate(`/stock/${results[0].symbol}`);
      }
    } catch (error) {
      console.error('Search error:', error);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-96">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div className="space-y-8">
      <div className="bg-gradient-to-r from-blue-600 to-blue-800 rounded-lg shadow-lg p-8 text-white">
        <h1 className="text-4xl font-bold mb-4">Real-Time Stock Market Data</h1>
        <p className="text-xl mb-6 text-blue-100">
          Track stocks, indices, and market trends with our free API
        </p>
        <SearchBar onSearch={handleSearch} />
      </div>

      <MarketOverview indices={indices.slice(0, 6)} />

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="bg-white rounded-lg shadow p-6">
          <div className="flex items-center space-x-2 mb-4">
            <TrendingUp className="h-6 w-6 text-green-600" />
            <h2 className="text-xl font-bold text-gray-900">Top Gainers</h2>
          </div>
          <div className="space-y-3">
            {gainers.map((stock) => (
              <div
                key={stock.symbol}
                className="flex justify-between items-center p-3 hover:bg-gray-50 rounded cursor-pointer transition"
                onClick={() => navigate(`/stock/${stock.symbol}`)}
              >
                <div>
                  <div className="font-semibold text-gray-900">{stock.symbol}</div>
                  <div className="text-sm text-gray-600">{stock.name}</div>
                </div>
                <div className="text-right">
                  <div className="font-semibold">${stock.price}</div>
                  <div className="text-sm text-green-600">{stock.percentChange}</div>
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className="bg-white rounded-lg shadow p-6">
          <div className="flex items-center space-x-2 mb-4">
            <TrendingDown className="h-6 w-6 text-red-600" />
            <h2 className="text-xl font-bold text-gray-900">Top Losers</h2>
          </div>
          <div className="space-y-3">
            {losers.map((stock) => (
              <div
                key={stock.symbol}
                className="flex justify-between items-center p-3 hover:bg-gray-50 rounded cursor-pointer transition"
                onClick={() => navigate(`/stock/${stock.symbol}`)}
              >
                <div>
                  <div className="font-semibold text-gray-900">{stock.symbol}</div>
                  <div className="text-sm text-gray-600">{stock.name}</div>
                </div>
                <div className="text-right">
                  <div className="font-semibold">${stock.price}</div>
                  <div className="text-sm text-red-600">{stock.percentChange}</div>
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className="bg-white rounded-lg shadow p-6">
          <div className="flex items-center space-x-2 mb-4">
            <Activity className="h-6 w-6 text-blue-600" />
            <h2 className="text-xl font-bold text-gray-900">Most Active</h2>
          </div>
          <div className="space-y-3">
            {actives.map((stock) => (
              <div
                key={stock.symbol}
                className="flex justify-between items-center p-3 hover:bg-gray-50 rounded cursor-pointer transition"
                onClick={() => navigate(`/stock/${stock.symbol}`)}
              >
                <div>
                  <div className="font-semibold text-gray-900">{stock.symbol}</div>
                  <div className="text-sm text-gray-600">{stock.name}</div>
                </div>
                <div className="text-right">
                  <div className="font-semibold">${stock.price}</div>
                  <div className={`text-sm ${stock.percentChange.startsWith('+') ? 'text-green-600' : 'text-red-600'}`}>
                    {stock.percentChange}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-xl font-bold text-gray-900 mb-4">Sector Performance</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {sectors.map((sector) => (
            <div key={sector.sector} className="border border-gray-200 rounded-lg p-4">
              <h3 className="font-semibold text-gray-900 mb-3">{sector.sector}</h3>
              <div className="grid grid-cols-2 gap-2 text-sm">
                <div>
                  <div className="text-gray-600">Today</div>
                  <div className={`font-semibold ${sector.dayReturn.startsWith('+') ? 'text-green-600' : 'text-red-600'}`}>
                    {sector.dayReturn}
                  </div>
                </div>
                <div>
                  <div className="text-gray-600">YTD</div>
                  <div className={`font-semibold ${sector.ytdReturn.startsWith('+') ? 'text-green-600' : 'text-red-600'}`}>
                    {sector.ytdReturn}
                  </div>
                </div>
                <div>
                  <div className="text-gray-600">1 Year</div>
                  <div className={`font-semibold ${sector.yearReturn.startsWith('+') ? 'text-green-600' : 'text-red-600'}`}>
                    {sector.yearReturn}
                  </div>
                </div>
                <div>
                  <div className="text-gray-600">5 Year</div>
                  <div className={`font-semibold ${sector.fiveYearReturn.startsWith('+') ? 'text-green-600' : 'text-red-600'}`}>
                    {sector.fiveYearReturn}
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      <NewsSection news={news} />
    </div>
  );
};

export default HomePage;
