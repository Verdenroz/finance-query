import { useTranslation } from 'react-i18next';
import { Check, Github } from 'lucide-react';

const PricingSection = () => {
  const { t } = useTranslation();
  const plans = ['selfHosted', 'personal', 'enterprise'];

  return (
    <div className="bg-gray-50 py-16 px-4">
      <div className="max-w-7xl mx-auto">
        <div className="text-center mb-12">
          <h2 className="text-4xl font-bold text-gray-900 mb-4">{t('pricing.title')}</h2>
          <p className="text-xl text-gray-600">{t('pricing.subtitle')}</p>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
          {plans.map((plan, idx) => (
            <div key={plan} className={'bg-white rounded-lg shadow-lg overflow-hidden transform transition hover:scale-105 ' + (idx === 1 ? 'ring-2 ring-blue-600' : '')}>
              {idx === 1 && <div className="bg-blue-600 text-white text-center py-2 text-sm font-semibold">MOST POPULAR</div>}
              <div className="p-8">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="text-2xl font-bold text-gray-900">{t('pricing.' + plan + '.name')}</h3>
                  {plan === 'selfHosted' && <Github className="h-8 w-8 text-gray-700" />}
                </div>
                <div className="mb-4"><span className="text-4xl font-bold text-gray-900">{t('pricing.' + plan + '.price')}</span></div>
                <div className="text-sm text-gray-600 mb-2">{t('pricing.' + plan + '.requests', {defaultValue: ''})}</div>
                <p className="text-gray-600 mb-6">{t('pricing.' + plan + '.description')}</p>
                <button className={'w-full py-3 px-6 rounded-lg font-semibold transition ' + (idx === 1 ? 'bg-blue-600 text-white hover:bg-blue-700' : 'bg-gray-100 text-gray-900 hover:bg-gray-200')}>
                  {t('pricing.' + plan + '.button')}
                </button>
                <ul className="mt-6 space-y-3">
                  {[0,1,2,3,4].map(i => {
                    const f = t('pricing.' + plan + '.features.' + i, {defaultValue: ''});
                    return f ? <li key={i} className="flex items-start"><Check className="h-5 w-5 text-green-500 mr-2 flex-shrink-0 mt-0.5"/><span className="text-gray-700">{f}</span></li> : null;
                  })}
                </ul>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default PricingSection;
