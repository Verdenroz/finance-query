import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { TrendingUp, TrendingDown, Activity, BarChart3, Newspaper, Clock } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { getIndices, getMarketMovers, getSectors, getNews, searchSymbols, type MarketIndex, type MarketMover, type MarketSector, type News } from '../services/api';
import MarketOverview from '../components/MarketOverview';
import NewsSection from '../components/NewsSection';
import SearchBar from '../components/SearchBar';
import PricingSection from '../components/PricingSection';

const HomePage = () => {
  const { t } = useTranslation();
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
      const results = await Promise.allSettled([
        getIndices(),
        getMarketMovers('gainers', 50),
        getMarketMovers('losers', 50),
        getMarketMovers('actives', 50),
        getSectors(),
        getNews(),
      ]);

      if (results[0].status === 'fulfilled') setIndices(results[0].value);
      if (results[1].status === 'fulfilled') setGainers(results[1].value);
      if (results[2].status === 'fulfilled') setLosers(results[2].value);
      if (results[3].status === 'fulfilled') setActives(results[3].value);
      if (results[4].status === 'fulfilled') setSectors(results[4].value);
      if (results[5].status === 'fulfilled') setNews(results[5].value);

      setLoading(false);
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
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center">
          <div className="animate-spin rounded-full h-16 w-16 border-b-4 border-blue-600 mx-auto mb-4"></div>
          <p className="text-gray-600 text-lg">Loading market data...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-12">
      <div className="relative bg-gradient-to-br from-blue-600 via-blue-700 to-indigo-800 rounded-2xl shadow-2xl overflow-hidden">
        <div className="absolute inset-0 bg-black opacity-10"></div>
        <div className="absolute inset-0 bg-grid-pattern opacity-5"></div>
        <div className="relative z-10 p-12">
          <div className="max-w-4xl mx-auto text-center text-white mb-8">
            <h1 className="text-5xl md:text-6xl font-bold mb-6 leading-tight">
              {t('hero.title')}
            </h1>
            <p className="text-xl md:text-2xl mb-4 text-blue-100 font-light">
              {t('hero.subtitle')}
            </p>
            <p className="text-lg mb-8 text-blue-200 max-w-2xl mx-auto">
              {t('hero.description')}
            </p>
          </div>
          <div className="max-w-3xl mx-auto mb-8">
            <SearchBar onSearch={handleSearch} />
          </div>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-6 max-w-4xl mx-auto">
            {['realtime', 'comprehensive', 'free', 'opensource'].map((feature) => (
              <div key={feature} className="text-center backdrop-blur-sm bg-white bg-opacity-10 rounded-xl p-4 border border-white border-opacity-20">
                <div className="text-3xl font-bold mb-2">✓</div>
                <div className="text-sm font-medium">{t(`hero.features.${feature}`)}</div>
              </div>
            ))}
          </div>
        </div>
      </div>

      <div className="bg-gradient-to-b from-gray-50 to-white rounded-2xl p-8 shadow-lg">
        <div className="flex items-center space-x-3 mb-6">
          <BarChart3 className="h-8 w-8 text-blue-600" />
          <h2 className="text-3xl font-bold text-gray-900">{t('sections.overview')}</h2>
        </div>
        {indices.length > 0 ? (
          <MarketOverview indices={indices.slice(0, 6)} />
        ) : (
          <p className="text-gray-500 text-center py-8">No market data available</p>
        )}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {[
          { data: gainers, title: 'gainers', icon: TrendingUp, color: 'green' },
          { data: losers, title: 'losers', icon: TrendingDown, color: 'red' },
          { data: actives, title: 'actives', icon: Activity, color: 'blue' },
        ].map(({ data, title, icon: Icon, color }) => (
          <div key={title} className="bg-white rounded-2xl shadow-lg overflow-hidden hover:shadow-xl transition-all duration-300">
            <div className={`bg-gradient-to-r ${color === 'green' ? 'from-green-500 to-emerald-600' : color === 'red' ? 'from-red-500 to-rose-600' : 'from-blue-500 to-cyan-600'} p-4`}>
              <div className="flex items-center space-x-3 text-white">
                <Icon className="h-7 w-7" />
                <h2 className="text-2xl font-bold">{t(`sections.${title}`)}</h2>
              </div>
            </div>
            <div className="p-4">
              {data.length > 0 ? (
                <div className="space-y-2">
                  {data.slice(0, 10).map((stock) => (
                    <div key={stock.symbol} className="group flex justify-between items-center p-3 rounded-lg hover:bg-gray-50 cursor-pointer transition-all" onClick={() => navigate(`/stock/${stock.symbol}`)}>
                      <div className="flex-1">
                        <div className="font-bold text-gray-900 group-hover:text-blue-600 transition">{stock.symbol}</div>
                        <div className="text-sm text-gray-600 truncate">{stock.name}</div>
                      </div>
                      <div className="text-right">
                        <div className="font-semibold text-gray-900">${stock.price}</div>
                        <div className={`text-sm font-medium ${color === 'green' ? 'text-green-600' : color === 'red' ? 'text-red-600' : 'text-blue-600'}`}>
                          {stock.percentChange}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-gray-500 text-center py-8">No data available</p>
              )}
            </div>
          </div>
        ))}
      </div>

      <PricingSection />

      {sectors.length > 0 && (
        <div className="bg-white rounded-2xl shadow-lg p-8">
          <h2 className="text-3xl font-bold text-gray-900 mb-6">{t('sections.sectors')}</h2>
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b-2 border-gray-200">
                  <th className="text-left py-4 px-4 font-bold text-gray-700">Sector</th>
                  <th className="text-right py-4 px-4 font-bold text-gray-700">Today</th>
                  <th className="text-right py-4 px-4 font-bold text-gray-700">YTD</th>
                  <th className="text-right py-4 px-4 font-bold text-gray-700">1Y</th>
                  <th className="text-right py-4 px-4 font-bold text-gray-700">5Y</th>
                </tr>
              </thead>
              <tbody>
                {sectors.map((sector, idx) => (
                  <tr key={sector.sector} className={`border-b border-gray-100 hover:bg-gray-50 transition ${idx % 2 === 0 ? 'bg-gray-50' : ''}`}>
                    <td className="py-4 px-4 font-semibold text-gray-900">{sector.sector}</td>
                    <td className={`text-right py-4 px-4 font-medium ${sector.dayReturn.startsWith('-') ? 'text-red-600' : 'text-green-600'}`}>{sector.dayReturn}</td>
                    <td className={`text-right py-4 px-4 font-medium ${sector.ytdReturn.startsWith('-') ? 'text-red-600' : 'text-green-600'}`}>{sector.ytdReturn}</td>
                    <td className={`text-right py-4 px-4 font-medium ${sector.yearReturn.startsWith('-') ? 'text-red-600' : 'text-green-600'}`}>{sector.yearReturn}</td>
                    <td className={`text-right py-4 px-4 font-medium ${sector.fiveYearReturn.startsWith('-') ? 'text-red-600' : 'text-green-600'}`}>{sector.fiveYearReturn}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {news.length > 0 && (
        <div className="bg-gradient-to-br from-gray-900 to-gray-800 rounded-2xl shadow-2xl p-8 text-white">
          <div className="flex items-center space-x-3 mb-6">
            <Newspaper className="h-8 w-8" />
            <h2 className="text-3xl font-bold">{t('sections.news')}</h2>
          </div>
          <NewsSection news={news.slice(0, 6)} />
        </div>
      )}
    </div>
  );
};

export default HomePage;
