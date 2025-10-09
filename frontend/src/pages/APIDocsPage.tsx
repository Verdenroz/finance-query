import { Book, Zap, Shield } from 'lucide-react';
import ContactForm from '../components/ContactForm';

const APIDocsPage = () => {
  const endpoints = [
    {
      method: 'GET',
      path: '/v1/quotes',
      description: 'Get detailed quote data for one or more symbols',
      params: 'symbols=NVDA,AAPL,TSLA',
      example: `curl -X GET 'http://localhost:8000/v1/quotes?symbols=NVDA'`,
    },
    {
      method: 'GET',
      path: '/v1/historical',
      description: 'Get historical price data with customizable time ranges',
      params: 'symbol=NVDA&range=1mo&interval=1d',
      example: `curl -X GET 'http://localhost:8000/v1/historical?symbol=NVDA&range=1mo&interval=1d'`,
    },
    {
      method: 'GET',
      path: '/v1/indices',
      description: 'Get major world market indices',
      params: 'region=US (optional)',
      example: `curl -X GET 'http://localhost:8000/v1/indices'`,
    },
    {
      method: 'GET',
      path: '/v1/gainers',
      description: 'Get top gaining stocks',
      params: 'count=50',
      example: `curl -X GET 'http://localhost:8000/v1/gainers?count=50'`,
    },
    {
      method: 'GET',
      path: '/v1/losers',
      description: 'Get top losing stocks',
      params: 'count=50',
      example: `curl -X GET 'http://localhost:8000/v1/losers?count=50'`,
    },
    {
      method: 'GET',
      path: '/v1/actives',
      description: 'Get most active stocks',
      params: 'count=50',
      example: `curl -X GET 'http://localhost:8000/v1/actives?count=50'`,
    },
    {
      method: 'GET',
      path: '/v1/sectors',
      description: 'Get sector performance summary',
      params: 'None',
      example: `curl -X GET 'http://localhost:8000/v1/sectors'`,
    },
    {
      method: 'GET',
      path: '/v1/news',
      description: 'Get financial news (general or for specific symbol)',
      params: 'symbol=NVDA (optional)',
      example: `curl -X GET 'http://localhost:8000/v1/news?symbol=NVDA'`,
    },
    {
      method: 'GET',
      path: '/v1/search',
      description: 'Search for securities',
      params: 'query=NVIDIA',
      example: `curl -X GET 'http://localhost:8000/v1/search?query=NVIDIA'`,
    },
    {
      method: 'GET',
      path: '/v1/holders',
      description: 'Get institutional and insider holder information',
      params: 'symbol=NVDA',
      example: `curl -X GET 'http://localhost:8000/v1/holders?symbol=NVDA'`,
    },
    {
      method: 'GET',
      path: '/v1/financials',
      description: 'Get financial statements',
      params: 'symbol=NVDA&period=annual',
      example: `curl -X GET 'http://localhost:8000/v1/financials?symbol=NVDA&period=annual'`,
    },
  ];

  return (
    <div className="space-y-8">
      <div className="bg-gradient-to-r from-blue-600 to-blue-800 rounded-lg shadow-lg p-8 text-white">
        <h1 className="text-4xl font-bold mb-4">API Documentation</h1>
        <p className="text-xl text-blue-100">
          Free and Open-Source Financial Data API
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="bg-white rounded-lg shadow p-6">
          <Zap className="h-10 w-10 text-blue-600 mb-3" />
          <h3 className="text-lg font-semibold text-gray-900 mb-2">Fast & Reliable</h3>
          <p className="text-gray-600">
            Built with FastAPI and optimized for high performance with intelligent caching
          </p>
        </div>
        <div className="bg-white rounded-lg shadow p-6">
          <Shield className="h-10 w-10 text-blue-600 mb-3" />
          <h3 className="text-lg font-semibold text-gray-900 mb-2">Free to Use</h3>
          <p className="text-gray-600">
            No registration required for basic usage. Rate limits apply to ensure fair access
          </p>
        </div>
        <div className="bg-white rounded-lg shadow p-6">
          <Book className="h-10 w-10 text-blue-600 mb-3" />
          <h3 className="text-lg font-semibold text-gray-900 mb-2">Open Source</h3>
          <p className="text-gray-600">
            MIT Licensed and available on GitHub. Deploy your own instance if needed
          </p>
        </div>
      </div>

      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-2xl font-bold text-gray-900 mb-4">Getting Started</h2>
        <div className="space-y-4">
          <div>
            <h3 className="text-lg font-semibold text-gray-900 mb-2">Base URL</h3>
            <code className="block bg-gray-50 p-3 rounded text-sm text-gray-900">
              http://localhost:8000
            </code>
            <p className="text-sm text-gray-600 mt-2">
              Replace with your deployed domain when using in production
            </p>
          </div>

          <div>
            <h3 className="text-lg font-semibold text-gray-900 mb-2">Authentication (Optional)</h3>
            <p className="text-gray-600 mb-2">
              Add an <code className="bg-gray-100 px-2 py-1 rounded">x-api-key</code> header for higher rate limits:
            </p>
            <code className="block bg-gray-50 p-3 rounded text-sm text-gray-900">
              curl -H "x-api-key: your-api-key" http://localhost:8000/v1/quotes?symbols=NVDA
            </code>
          </div>

          <div>
            <h3 className="text-lg font-semibold text-gray-900 mb-2">Rate Limits</h3>
            <p className="text-gray-600">
              Without an API key: 8,000 requests per day per IP address
            </p>
          </div>
        </div>
      </div>

      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-2xl font-bold text-gray-900 mb-6">API Endpoints</h2>
        <div className="space-y-6">
          {endpoints.map((endpoint, index) => (
            <div key={index} className="border border-gray-200 rounded-lg p-4">
              <div className="flex items-center space-x-3 mb-3">
                <span className="bg-blue-100 text-blue-800 text-xs font-semibold px-2.5 py-0.5 rounded">
                  {endpoint.method}
                </span>
                <code className="text-sm font-mono text-gray-900">{endpoint.path}</code>
              </div>
              <p className="text-gray-600 mb-3">{endpoint.description}</p>
              <div className="space-y-2">
                <div>
                  <span className="text-sm font-semibold text-gray-700">Parameters:</span>
                  <code className="ml-2 text-sm bg-gray-50 px-2 py-1 rounded">{endpoint.params}</code>
                </div>
                <div>
                  <span className="text-sm font-semibold text-gray-700">Example:</span>
                  <code className="block mt-1 bg-gray-900 text-green-400 p-3 rounded text-sm overflow-x-auto">
                    {endpoint.example}
                  </code>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-2xl font-bold text-gray-900 mb-4">WebSocket Support</h2>
        <p className="text-gray-600 mb-4">
          Real-time data streaming is available via WebSocket connections:
        </p>
        <div className="space-y-4">
          <div>
            <h3 className="text-lg font-semibold text-gray-900 mb-2">/quotes</h3>
            <p className="text-gray-600 mb-2">Stream real-time quote updates</p>
            <code className="block bg-gray-900 text-green-400 p-3 rounded text-sm overflow-x-auto">
              {`const ws = new WebSocket('ws://localhost:8000/quotes');
ws.onopen = () => ws.send('TSLA');
ws.onmessage = (event) => console.log(JSON.parse(event.data));`}
            </code>
          </div>
        </div>
      </div>

      <div className="bg-blue-50 border border-blue-200 rounded-lg p-6">
        <h2 className="text-xl font-bold text-blue-900 mb-3">Open Source & License</h2>
        <p className="text-blue-800 mb-4">
          FinanceQuery is open-source software released under the MIT License. You are free to use,
          modify, and distribute this software. The project is developed for educational purposes and
          provides free financial data without any corporate affiliation.
        </p>
        <p className="text-sm text-blue-700">
          This project complies with Google Ads policies as a free, open-source tool without business
          registration requirements. Financial data is provided for informational purposes only and should
          not be considered financial advice.
        </p>
      </div>

      <div className="mt-12">
        <ContactForm />
      </div>
    </div>
  );
};

export default APIDocsPage;
