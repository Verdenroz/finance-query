# API修复文档

## 修复内容

### 1. Market Movers参数修正 ✅

**问题：** 前端使用count=10，但API只支持25/50/100

**后端配置（已正确）：**
```python
# src/models/marketmover.py
class MoverCount(Enum):
    TWENTY_FIVE = "25"
    FIFTY = "50"
    HUNDRED = "100"
```

**前端修复：**
```typescript
// 之前
getMarketMovers('gainers', 10)  // ❌ 不支持
getMarketMovers('losers', 10)   // ❌ 不支持
getMarketMovers('actives', 10)  // ❌ 不支持

// 现在
getMarketMovers('gainers', 50)  // ✅ 获取50条
getMarketMovers('losers', 50)   // ✅ 获取50条
getMarketMovers('actives', 50)  // ✅ 获取50条

// 显示前10条
gainers.slice(0, 10).map(...)
losers.slice(0, 10).map(...)
actives.slice(0, 10).map(...)
```

**好处：**
- 获取更多数据，减少API调用
- 可以随时调整显示数量，无需重新请求
- 符合API规范

### 2. Stock Quote API分析 ✅

**当前实现：**
```python
# src/services/quotes/get_quotes.py
@retry(scrape_quotes)
async def get_quotes(finance_client, symbols):
    return await fetch_quotes(finance_client, symbols)
```

**工作流程：**
1. 首先尝试Yahoo Finance API
2. 如果失败，自动fallback到scraping
3. 使用@retry装饰器自动重试
4. 并行处理多个symbol，提高效率

**可能的失败原因：**

#### A. Symbol不存在
```python
# Yahoo返回404
# 示例：AAPLE (拼写错误)
# 处理：客户端需要验证symbol
```

#### B. Yahoo Finance限流
```python
# Yahoo返回429
# 处理：已有retry机制和fallback
```

#### C. Symbol类型不支持
```python
# 某些市场的股票可能不在Yahoo Finance
# 例如：一些场外交易(OTC)股票
```

## Yahoo Finance API端点

根据代码实现，使用的端点：

### 1. 详细Quote
```
GET https://query2.finance.yahoo.com/v10/finance/quoteSummary/{symbol}
Parameters:
  - crumb: [auth token]
  - modules: assetProfile,price,summaryDetail,defaultKeyStatistics,
             calendarEvents,quoteUnadjustedPerformanceOverview
```

**返回数据：**
- 价格数据（当前、开盘、最高、最低）
- 市场数据（市值、成交量）
- 基本面（PE、EPS、股息）
- 公司信息（行业、描述）
- 业绩数据（各时间段回报）

### 2. 简单Quote（批量）
```
GET https://query1.finance.yahoo.com/v7/finance/quote
Parameters:
  - crumb: [auth token]
  - symbols: AAPL,MSFT,GOOGL (逗号分隔)
```

**返回数据：**
- 简化的价格信息
- 支持批量查询（一次多个symbol）
- 响应更快

### 3. 历史数据
```
GET https://query1.finance.yahoo.com/v8/finance/chart/{symbol}
Parameters:
  - crumb: [auth token]
  - interval: 1d, 1wk, 1mo等
  - range: 1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max
```

### 4. Market Movers
```
# API方式（主要）
GET https://query1.finance.yahoo.com/v1/finance/screener/predefined/saved
Parameters:
  - count: 25, 50, 100
  - formatted: true
  - scrIds: MOST_ACTIVES | DAY_GAINERS | DAY_LOSERS

# Scraping方式（fallback）
GET https://finance.yahoo.com/markets/stocks/most-active/?start=0&count=50
GET https://finance.yahoo.com/markets/stocks/gainers/?start=0&count=50
GET https://finance.yahoo.com/markets/stocks/losers/?start=0&count=50
```

### 5. Search
```
GET https://query2.finance.yahoo.com/v1/finance/search
Parameters:
  - crumb: [auth token]
  - q: [search query]
```

## 错误处理

### 当前机制

**1. 多层Fallback**
```python
@retry(scrape_quotes)  # fallback函数
async def get_quotes(finance_client, symbols):
    return await fetch_quotes(finance_client, symbols)  # 主函数
```

**2. HTTP错误码映射**
```python
401 → "Yahoo auth failed"
404 → "Symbol not found"
429 → "Rate limit exceeded"
500 → "Server error"
```

**3. 自动重试**
- API失败自动切换到scraping
- 网络错误自动重试
- 并行处理减少单点失败影响

### 建议改进

**1. Symbol验证**
```typescript
// 前端添加symbol格式验证
function validateSymbol(symbol: string): boolean {
  // 1-5个大写字母
  return /^[A-Z]{1,5}$/.test(symbol);
}

// 调用前验证
if (!validateSymbol(symbol)) {
  throw new Error('Invalid symbol format');
}
```

**2. 错误提示优化**
```typescript
// 区分不同错误类型
try {
  const quote = await getQuotes([symbol]);
} catch (error) {
  if (error.status === 404) {
    showError('股票代码不存在，请检查输入');
  } else if (error.status === 429) {
    showError('请求过于频繁，请稍后再试');
  } else {
    showError('获取数据失败，请重试');
  }
}
```

**3. 缓存机制**
```python
# 已有Redis缓存，可以调整TTL
@cache(ttl=300)  # 5分钟缓存
async def get_quotes(...):
    ...
```

## 测试建议

### 1. 测试不同类型的Symbol

**美股：**
```bash
curl "http://localhost:8080/v1/quotes?symbols=AAPL,MSFT,GOOGL"
```

**ETF：**
```bash
curl "http://localhost:8080/v1/quotes?symbols=SPY,QQQ,VOO"
```

**加密货币相关：**
```bash
curl "http://localhost:8080/v1/quotes?symbols=BTC-USD,ETH-USD"
```

**外汇：**
```bash
curl "http://localhost:8080/v1/quotes?symbols=EURUSD=X,GBPUSD=X"
```

### 2. 测试错误情况

**不存在的Symbol：**
```bash
curl "http://localhost:8080/v1/quotes?symbols=AAPLE"
# 应返回404或空数组
```

**批量查询：**
```bash
curl "http://localhost:8080/v1/quotes?symbols=AAPL,INVALID,MSFT"
# 应返回AAPL和MSFT，跳过INVALID
```

### 3. 测试Market Movers

**Gainers：**
```bash
curl "http://localhost:8080/v1/gainers?count=50"
```

**Losers：**
```bash
curl "http://localhost:8080/v1/losers?count=50"
```

**Actives：**
```bash
curl "http://localhost:8080/v1/actives?count=50"
```

**无效参数：**
```bash
curl "http://localhost:8080/v1/gainers?count=10"
# 应返回422错误
```

## 常见问题

### Q: 为什么某些股票查不到？

**A:** 可能原因：
1. Symbol拼写错误（如AAPLE而不是AAPL）
2. 股票已退市或合并
3. 非美股，Yahoo Finance可能不支持
4. OTC市场的小盘股

**解决：**
- 先用search接口搜索正确的symbol
- 检查股票所在市场
- 某些市场需要后缀（如.L表示伦敦）

### Q: 为什么API有时很慢？

**A:** 可能原因：
1. Yahoo Finance服务器响应慢
2. 并发请求过多
3. 网络延迟

**解决：**
- 使用simple-quotes而不是quotes（更快）
- 启用Redis缓存
- 减少并发请求数
- 使用批量接口而不是单个查询

### Q: 为什么有时返回空数据？

**A:** 可能原因：
1. 市场闭市，某些字段可能为null
2. Yahoo Finance数据缺失
3. 爬虫fallback失败

**解决：**
- 检查response_model_exclude_none配置
- 前端做好null值处理
- 查看日志确认是否fallback成功

## API兼容性

### Yahoo Finance API限制

**1. 需要认证**
- 需要cookies和crumb token
- Token定期过期需要刷新
- 项目已实现自动刷新机制

**2. 速率限制**
- 每分钟约100-200请求
- 超限返回429错误
- 项目有retry和fallback机制

**3. 数据延迟**
- 实时数据可能有15-20分钟延迟
- 盘前盘后数据可能不准确
- 某些小盘股数据更新慢

### 与官方文档对比

项目实现了更稳定的方案：
- **双路径**：API + Scraping
- **自动重试**：失败自动fallback
- **缓存**：减少对Yahoo的请求
- **并行**：提高处理速度

## 部署后验证

### 1. 基本功能测试
```bash
# 市场指数
curl "https://xiaocunan.com/v1/indices"

# Market Movers（使用正确参数）
curl "https://xiaocunan.com/v1/gainers?count=50"
curl "https://xiaocunan.com/v1/losers?count=50"
curl "https://xiaocunan.com/v1/actives?count=50"

# 股票Quote
curl "https://xiaocunan.com/v1/quotes?symbols=AAPL,MSFT"

# 搜索
curl "https://xiaocunan.com/v1/search?query=apple"
```

### 2. 前端验证
```bash
# 访问首页
https://xiaocunan.com/

# 检查控制台无错误
# 检查Market Movers显示正常
# 测试搜索功能
# 查看股票详情页
```

### 3. 监控日志
```bash
# 检查错误日志
docker logs financequery-backend | grep ERROR

# 检查API调用
docker logs financequery-backend | grep "Yahoo Finance"

# 检查fallback使用情况
docker logs financequery-backend | grep "scrape"
```

## 总结

### 已修复 ✅
1. Market Movers count参数：10 → 50
2. 前端显示：slice(0, 10)只显示前10条
3. 错误处理机制已完善
4. Fallback机制已实现

### 不需要修复 ✅
1. Quote API实现正确
2. 已有完善的错误处理
3. 已有retry和fallback
4. 代码架构清晰合理

### 建议增强
1. 前端添加symbol验证
2. 优化错误提示
3. 添加loading状态
4. 考虑数据预加载

### 已知限制
1. 某些小盘股可能查不到
2. 非美股支持有限
3. 数据有延迟（15-20分钟）
4. Yahoo限流问题（已有fallback）

**结论：** API实现健壮，主要问题是前端参数不匹配，已修复。
