# FinanceQuery

FinanceQuery is a free and open-source API to query financial data. It provides endpoints to get quotes, historical
prices, indices, market movers, similar stocks, finance news, indicators, search, and sectors. Data is acquired through
web scraping and the unofficial Yahoo Finance API. It is the successor to
the [GoogleFinanceAPI](https://github.com/Verdenroz/GoogleFinanceAPI).

## Documentation

[Documentation](https://financequery.apidocumentation.com/)

## Run Locally

Clone the project

```bash
  git clone https://github.com/Verdenroz/finance-query.git
```

Go to the project directory

```bash
  cd finance-query
```

Install dependencies

```bash
 pip install -r requirements.txt
```

Cythonize files

```bash
  python setup.py build_ext --inplace
```

Start the server

```bash
  python -m uvicorn src.main:app --reload
```

## Deployment

#### AWS Lambda

- Follow
  the [AWS Lambda Deployment Guide](https://docs.aws.amazon.com/lambda/latest/dg/python-image.html#python-image-instructions)
- Remember to add the environment variables to the Lambda function
- Alternatively use the [AWS Deployment Workflow](.github/workflows/aws-deploy.yml), providing repository secrets
  for `AWS_SECRET_ID` and `AWS_SECRET_KEY`.
    - Also edit the `AWS_REGION`, `ECR_REPOSITORY`, and `FUNCTION_NAME` in the workflow file

#### Render

- Follow the [Render Deployment Guide](https://render.com/docs/deploy-fastapi)
- The deployment should use the `Dockerfile` file in the repository
- Be sure to override the CMD in the Dockerfile in your Render project settings
  to `python -m uvicorn src.main:app --host 0.0.0.0 --port $PORT`
- Alternatively use the [Render Deployment Workflow](.github/workflows/render-deploy.yml), providing repository secrets
  for `RENDER_DEPLOY_HOOK_URL`.
    - The deploy hook url can be found in the settings of your Render project

## Usage/Examples

The exposed endpoints to the API is

- https://43pk30s7aj.execute-api.us-east-2.amazonaws.com/prod
- https://finance-query.onrender.com

There are two workflows that will automatically deploy to render and AWS, but they will require repository secrets
for `AWS_SECRET_ID`, `AWS_SECRET_KEY`, and `RENDER_DEPLOY_HOOK_URL`. Quite frankly, render is easier to work with since
it enables the websockets, but will require the paid Starter Plan as this API requires extensive memory. If you are
tight on cash, consider Lambda.

An `x-api-key` header can be added if you have enabled security and rate limiting. If a key is not provided, or an
invalid key is used, a rate limit of 2000 requests/day is applied to the request's ip address.

> If you are deploying this for yourself, you can create your own admin key which will not be rate limited. See
> the [.env template](.env.template).
> Again, remember the websockets above are not available through Lambda. If you deploy to Render instead, you will be
> able to connect to the websockets through a request that looks like `wss://finance-query.onrender.com/...`

### Example REST Request

```bash
# Get detailed quote for NVIDIA stock
curl -X GET 'https://finance-query.onrender.com/v1/quotes?symbols=nvda' \
  -H 'x-api-key: your-api-key'
```

#### Response

```json
[
  {
    "symbol": "NVDA",
    "name": "NVIDIA Corporation",
    "price": "120.15",
    "afterHoursPrice": "121.60",
    "change": "-11.13",
    "percentChange": "-8.48%",
    "open": "135.00",
    "high": "135.00",
    "low": "120.01",
    "yearHigh": "153.13",
    "yearLow": "75.61",
    "volume": 432855617,
    "avgVolume": 238908833,
    "marketCap": "2.94T",
    "pe": "40.87",
    "dividend": "0.04",
    "yield": "0.03%",
    "exDividend": "Mar 12, 2025",
    "earningsDate": "May 20, 2025 - May 26, 2025",
    "lastDividend": "0.01",
    "sector": "Technology",
    "industry": "Semiconductors",
    "about": "NVIDIA Corporation provides graphics and compute and networking solutions in the United States, Taiwan, China, Hong Kong, and internationally. The Graphics segment offers GeForce GPUs for gaming and PCs, the GeForce NOW game streaming service and related infrastructure, and solutions for gaming platforms; Quadro/NVIDIA RTX GPUs for enterprise workstation graphics; virtual GPU or vGPU software for cloud-based visual and virtual computing; automotive platforms for infotainment systems; and Omniverse software for building and operating metaverse and 3D internet applications. The Compute & Networking segment comprises Data Center computing platforms and end-to-end networking platforms, including Quantum for InfiniBand and Spectrum for Ethernet; NVIDIA DRIVE automated-driving platform and automotive development agreements; Jetson robotics and other embedded platforms; NVIDIA AI Enterprise and other software; and DGX Cloud software and services. The company's products are used in gaming, professional visualization, data center, and automotive markets. It sells its products to original equipment manufacturers, original device manufacturers, system integrators and distributors, independent software vendors, cloud service providers, consumer internet companies, add-in board manufacturers, distributors, automotive manufacturers and tier-1 automotive suppliers, and other ecosystem participants. It has a strategic collaboration with IQVIA to help realize the potential of AI in healthcare and life sciences. NVIDIA Corporation was incorporated in 1993 and is headquartered in Santa Clara, California.",
    "fiveDaysReturn": "-14.25%",
    "oneMonthReturn": "1.46%",
    "threeMonthReturn": "-11.22%",
    "sixMonthReturn": "-6.35%",
    "ytdReturn": "-10.53%",
    "yearReturn": "52.67%",
    "threeYearReturn": "397.37%",
    "fiveYearReturn": "1,802.61%",
    "tenYearReturn": "21,686.04%",
    "maxReturn": "274,528.59%",
    "logo": "https://logo.clearbit.com/https://www.nvidia.com"
  }
]
```

### Example WebSocket Connection

```javascript
// Connect to WebSocket and subscribe to Tesla stock updates
const ws = new WebSocket('wss://finance-query.onrender.com/quotes');

ws.onopen = () => {
    console.log('Connected to WebSocket');
    // Send symbol to subscribe to updates
    ws.send('TSLA');
};

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    console.log('Received update:', data);
};
```

#### Sample WebSocket Message

```json
[
  {
    "symbol": "TSLA",
    "name": "Tesla, Inc.",
    "price": "398.09",
    "change": "+0.94",
    "percentChange": "+0.24%",
    "logo": "https://logo.clearbit.com/https://www.tesla.com"
  }
]
```

## Feedback

*As most data is scraped, some endpoints may break*

If something is not working or if you have any suggestions, contact me at harveytseng2@gmail.com

## License

[MIT](https://opensource.org/license/MIT)
