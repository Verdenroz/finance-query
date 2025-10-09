import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import HomePage from './pages/HomePage';
import StockDetailPage from './pages/StockDetailPage';
import APIDocsPage from './pages/APIDocsPage';
import Layout from './components/Layout';

function App() {
  return (
    <Router>
      <Layout>
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/stock/:symbol" element={<StockDetailPage />} />
          <Route path="/api-docs" element={<APIDocsPage />} />
        </Routes>
      </Layout>
    </Router>
  );
}

export default App;
