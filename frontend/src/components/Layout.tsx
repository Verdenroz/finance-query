import type { ReactNode } from 'react';
import { Link } from 'react-router-dom';
import { TrendingUp } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import LanguageSwitcher from './LanguageSwitcher';

interface LayoutProps {
  children: ReactNode;
}

const Layout = ({ children }: LayoutProps) => {
  const { t } = useTranslation();
  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white border-b border-gray-200 sticky top-0 z-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            <Link to="/" className="flex items-center space-x-2">
              <TrendingUp className="h-8 w-8 text-blue-600" />
              <span className="text-2xl font-bold text-gray-900">FinanceQuery</span>
            </Link>
            <nav className="flex items-center space-x-8">
              <Link to="/" className="text-gray-700 hover:text-blue-600 font-medium transition">
                {t('nav.home')}
              </Link>
              <Link to="/api-docs" className="text-gray-700 hover:text-blue-600 font-medium transition">
                {t('nav.apiDocs')}
              </Link>
              <LanguageSwitcher />
            </nav>
          </div>
        </div>
      </header>
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {children}
      </main>
      <footer className="bg-white border-t border-gray-200 mt-16">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <TrendingUp className="h-6 w-6 text-blue-600" />
                <span className="text-xl font-bold text-gray-900">FinanceQuery</span>
              </div>
              <div className="text-sm text-gray-600">
                Open-Source Financial Data API
              </div>
            </div>
            <div className="border-t border-gray-200 pt-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <h3 className="font-semibold text-gray-900 mb-2">About</h3>
                  <p className="text-sm text-gray-600 leading-relaxed">
                    FinanceQuery is a free, open-source API providing real-time stock data, market information,
                    and financial news. This project is released under the MIT License and is available for anyone
                    to use, modify, and distribute.
                  </p>
                </div>
                <div>
                  <h3 className="font-semibold text-gray-900 mb-2">Free API Access</h3>
                  <p className="text-sm text-gray-600 leading-relaxed">
                    Developers can freely integrate our API into their applications. Visit the{' '}
                    <Link to="/api-docs" className="text-blue-600 hover:underline">API Documentation</Link>
                    {' '}page to get started. No registration or API key required for basic usage.
                  </p>
                </div>
              </div>
            </div>
            <div className="border-t border-gray-200 pt-4 flex flex-col md:flex-row justify-between items-center space-y-2 md:space-y-0">
              <div className="text-sm text-gray-600">
                Licensed under the{' '}
                <a
                  href="https://opensource.org/licenses/MIT"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-blue-600 hover:underline"
                >
                  MIT License
                </a>
              </div>
              <div className="text-sm text-gray-600">
                © 2025 FinanceQuery. All financial data is provided for informational purposes only.
              </div>
            </div>
            <div className="border-t border-gray-200 pt-4">
              <p className="text-xs text-gray-500 text-center">
                This is an open-source educational tool. Data accuracy is not guaranteed. Not financial advice.
                No corporate entity or business affiliation. Compliant with Google Ads policies for free, open-source tools.
              </p>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
};

export default Layout;
