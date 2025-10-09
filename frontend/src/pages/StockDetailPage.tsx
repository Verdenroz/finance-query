import { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { getQuotes, getHistoricalData, getNews, getHolders, getFinancials, type Quote, type News, type Holders, type Financials, type HistoricalDataPoint } from '../services/api';
import StockHeader from '../components/StockHeader';
import StockChart from '../components/StockChart';
import StockTabs from '../components/StockTabs';

const StockDetailPage = () => {
  const { symbol } = useParams<{ symbol: string }>();
  const [quote, setQuote] = useState<Quote | null>(null);
  const [historicalData, setHistoricalData] = useState<Record<string, HistoricalDataPoint>>({});
  const [news, setNews] = useState<News[]>([]);
  const [holders, setHolders] = useState<Holders | null>(null);
  const [financials, setFinancials] = useState<Financials | null>(null);
  const [loading, setLoading] = useState(true);
  const [timeRange, setTimeRange] = useState('1mo');
  const [interval, setInterval] = useState('1d');

  useEffect(() => {
    const fetchData = async () => {
      if (!symbol) return;

      try {
        setLoading(true);
        const [quoteData, historicalDataResult, newsData, holdersData, financialsData] = await Promise.all([
          getQuotes([symbol]),
          getHistoricalData(symbol, timeRange, interval),
          getNews(symbol),
          getHolders(symbol).catch(() => null),
          getFinancials(symbol).catch(() => null),
        ]);

        if (quoteData.length > 0) {
          setQuote(quoteData[0]);
        }
        setHistoricalData(historicalDataResult);
        setNews(newsData);
        setHolders(holdersData);
        setFinancials(financialsData);
      } catch (error) {
        console.error('Error fetching stock data:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, [symbol, timeRange, interval]);

  if (loading || !quote) {
    return (
      <div className="flex items-center justify-center min-h-96">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <StockHeader quote={quote} />
      <StockChart
        data={historicalData}
        symbol={symbol || ''}
        timeRange={timeRange}
        interval={interval}
        onTimeRangeChange={setTimeRange}
        onIntervalChange={setInterval}
      />
      <StockTabs
        quote={quote}
        news={news}
        holders={holders}
        financials={financials}
      />
    </div>
  );
};

export default StockDetailPage;
