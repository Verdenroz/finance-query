import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import type { HistoricalDataPoint } from '../services/api';

interface StockChartProps {
  data: Record<string, HistoricalDataPoint>;
  symbol: string;
  timeRange: string;
  interval: string;
  onTimeRangeChange: (range: string) => void;
  onIntervalChange: (interval: string) => void;
}

const StockChart = ({ data, symbol, timeRange, onTimeRangeChange, onIntervalChange }: StockChartProps) => {
  const chartData = Object.entries(data).map(([date, point]) => ({
    date: new Date(date).toLocaleDateString(),
    price: point.close,
    volume: point.volume,
  }));

  const timeRanges = [
    { label: '1D', value: '1d', interval: '1m' },
    { label: '5D', value: '5d', interval: '5m' },
    { label: '1M', value: '1mo', interval: '1d' },
    { label: '3M', value: '3mo', interval: '1d' },
    { label: '6M', value: '6mo', interval: '1d' },
    { label: '1Y', value: '1y', interval: '1d' },
    { label: '5Y', value: '5y', interval: '1wk' },
  ];

  const handleRangeChange = (range: string, defaultInterval: string) => {
    onTimeRangeChange(range);
    onIntervalChange(defaultInterval);
  };

  return (
    <div className="bg-white rounded-lg shadow p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-bold text-gray-900">{symbol} Price Chart</h2>
        <div className="flex space-x-2">
          {timeRanges.map((range) => (
            <button
              key={range.value}
              onClick={() => handleRangeChange(range.value, range.interval)}
              className={`px-3 py-1 rounded text-sm font-medium transition ${
                timeRange === range.value
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
            >
              {range.label}
            </button>
          ))}
        </div>
      </div>

      <ResponsiveContainer width="100%" height={400}>
        <LineChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
          <XAxis
            dataKey="date"
            stroke="#6b7280"
            tick={{ fontSize: 12 }}
            tickFormatter={(value) => {
              if (chartData.length > 50) {
                return '';
              }
              return value;
            }}
          />
          <YAxis
            stroke="#6b7280"
            tick={{ fontSize: 12 }}
            domain={['auto', 'auto']}
            tickFormatter={(value) => `$${value.toFixed(2)}`}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: '#fff',
              border: '1px solid #e5e7eb',
              borderRadius: '0.375rem',
            }}
            formatter={(value: number) => [`$${value.toFixed(2)}`, 'Price']}
          />
          <Line
            type="monotone"
            dataKey="price"
            stroke="#2563eb"
            strokeWidth={2}
            dot={false}
            activeDot={{ r: 4 }}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
};

export default StockChart;
