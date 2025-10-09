import type { News } from '../services/api';
import { Newspaper } from 'lucide-react';

interface NewsSectionProps {
  news: News[];
  title?: string;
}

const NewsSection = ({ news, title = 'Latest News' }: NewsSectionProps) => {
  return (
    <div className="bg-white rounded-lg shadow p-6">
      <div className="flex items-center space-x-2 mb-4">
        <Newspaper className="h-6 w-6 text-blue-600" />
        <h2 className="text-xl font-bold text-gray-900">{title}</h2>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {news.slice(0, 10).map((item, index) => (
          <a
            key={index}
            href={item.link}
            target="_blank"
            rel="noopener noreferrer"
            className="flex space-x-3 p-3 hover:bg-gray-50 rounded transition group"
          >
            {item.thumbnail && (
              <img
                src={item.thumbnail}
                alt=""
                className="w-24 h-24 object-cover rounded flex-shrink-0"
              />
            )}
            <div className="flex-1 min-w-0">
              <h3 className="font-semibold text-gray-900 group-hover:text-blue-600 line-clamp-2 transition">
                {item.title}
              </h3>
              <div className="flex items-center space-x-2 mt-2 text-sm text-gray-600">
                <span>{item.publisher}</span>
                <span>•</span>
                <span>{item.timestamp}</span>
              </div>
            </div>
          </a>
        ))}
      </div>
    </div>
  );
};

export default NewsSection;
