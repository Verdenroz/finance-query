import axios from 'axios';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8000';

const api = axios.create({
  baseURL: API_BASE_URL,
  timeout: 30000,
});

export interface Quote {
  symbol: string;
  name: string;
  price: string;
  afterHoursPrice?: string;
  change: string;
  percentChange: string;
  open?: string;
  high?: string;
  low?: string;
  yearHigh?: string;
  yearLow?: string;
  volume?: number;
  avgVolume?: number;
  marketCap?: string;
  pe?: string;
  dividend?: string;
  yield?: string;
  exDividend?: string;
  earningsDate?: string;
  sector?: string;
  industry?: string;
  about?: string;
  logo?: string;
  fiveDaysReturn?: string;
  oneMonthReturn?: string;
  threeMonthReturn?: string;
  sixMonthReturn?: string;
  ytdReturn?: string;
  yearReturn?: string;
  threeYearReturn?: string;
  fiveYearReturn?: string;
}

export interface SimpleQuote {
  symbol: string;
  name: string;
  price: string;
  change: string;
  percentChange: string;
  logo?: string;
}

export interface HistoricalDataPoint {
  date: string;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}

export interface MarketMover {
  symbol: string;
  name: string;
  price: string;
  change: string;
  percentChange: string;
  logo?: string;
}

export interface MarketIndex {
  index: string;
  name: string;
  price: string;
  change: string;
  percentChange: string;
}

export interface MarketSector {
  sector: string;
  dayReturn: string;
  ytdReturn: string;
  yearReturn: string;
  threeYearReturn: string;
  fiveYearReturn: string;
}

export interface News {
  title: string;
  publisher: string;
  link: string;
  timestamp: string;
  thumbnail?: string;
}

export interface SearchResult {
  symbol: string;
  name: string;
  type: string;
  exchange?: string;
}

export interface Holders {
  institutionalOwnership?: number;
  insiderOwnership?: number;
  majorHolders?: Array<{
    name: string;
    shares: number;
    value: number;
    percentHeld: number;
  }>;
}

export interface Financials {
  symbol: string;
  annualReports?: any[];
  quarterlyReports?: any[];
}

export const getQuotes = async (symbols: string[]): Promise<Quote[]> => {
  const response = await api.get(`/v1/quotes?symbols=${symbols.join(',')}`);
  return response.data;
};

export const getSimpleQuotes = async (symbols: string[]): Promise<SimpleQuote[]> => {
  const response = await api.get(`/v1/simple-quotes?symbols=${symbols.join(',')}`);
  return response.data;
};

export const getHistoricalData = async (
  symbol: string,
  range: string,
  interval: string
): Promise<Record<string, HistoricalDataPoint>> => {
  const response = await api.get(`/v1/historical?symbol=${symbol}&range=${range}&interval=${interval}`);
  return response.data;
};

export const getMarketMovers = async (type: 'actives' | 'gainers' | 'losers', count: number = 50): Promise<MarketMover[]> => {
  const response = await api.get(`/v1/${type}?count=${count}`);
  return response.data;
};

export const getIndices = async (): Promise<MarketIndex[]> => {
  const response = await api.get('/v1/indices');
  return response.data;
};

export const getSectors = async (): Promise<MarketSector[]> => {
  const response = await api.get('/v1/sectors');
  return response.data;
};

export const getNews = async (symbol?: string): Promise<News[]> => {
  const url = symbol ? `/v1/news?symbol=${symbol}` : '/v1/news';
  const response = await api.get(url);
  return response.data;
};

export const searchSymbols = async (query: string): Promise<SearchResult[]> => {
  const response = await api.get(`/v1/search?query=${query}`);
  return response.data;
};

export const getHolders = async (symbol: string): Promise<Holders> => {
  const response = await api.get(`/v1/holders?symbol=${symbol}`);
  return response.data;
};

export const getFinancials = async (symbol: string, period: 'annual' | 'quarterly' = 'annual'): Promise<Financials> => {
  const response = await api.get(`/v1/financials?symbol=${symbol}&period=${period}`);
  return response.data;
};

export const getTechnicalIndicators = async (symbol: string, interval: string = '1d') => {
  const response = await api.get(`/v1/indicators?symbol=${symbol}&interval=${interval}`);
  return response.data;
};

export default api;
