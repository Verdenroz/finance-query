import { useState } from 'react';
import { Search } from 'lucide-react';

interface SearchBarProps {
  onSearch: (query: string) => void;
}

const SearchBar = ({ onSearch }: SearchBarProps) => {
  const [query, setQuery] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (query.trim()) {
      onSearch(query.trim());
    }
  };

  return (
    <form onSubmit={handleSubmit} className="max-w-2xl">
      <div className="relative">
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search stocks (e.g., AAPL, TSLA, NVDA)..."
          className="w-full px-4 py-3 pl-12 pr-4 text-gray-900 bg-white rounded-lg shadow-md focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <Search className="absolute left-4 top-3.5 h-5 w-5 text-gray-400" />
      </div>
    </form>
  );
};

export default SearchBar;
