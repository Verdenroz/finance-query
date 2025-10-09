import { useTranslation } from 'react-i18next';
import { Globe } from 'lucide-react';

const LanguageSwitcher = () => {
  const { i18n } = useTranslation();
  const languages = [
    { code: 'en', name: 'English' },
    { code: 'zh', name: '中文' },
    { code: 'ja', name: '日本語' },
    { code: 'de', name: 'Deutsch' },
  ];

  return (
    <div className="relative group">
      <button className="flex items-center space-x-2 px-3 py-2 rounded hover:bg-gray-100 transition">
        <Globe className="h-5 w-5" />
        <span className="hidden sm:inline">{languages.find(l => l.code === i18n.language)?.name || 'English'}</span>
      </button>
      <div className="absolute right-0 mt-2 w-40 bg-white rounded-lg shadow-lg opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all z-10">
        {languages.map(lang => (
          <button
            key={lang.code}
            onClick={() => i18n.changeLanguage(lang.code)}
            className={'block w-full text-left px-4 py-2 hover:bg-gray-100 ' + (i18n.language === lang.code ? 'bg-blue-50 text-blue-600 font-semibold' : '')}
          >
            {lang.name}
          </button>
        ))}
      </div>
    </div>
  );
};

export default LanguageSwitcher;
