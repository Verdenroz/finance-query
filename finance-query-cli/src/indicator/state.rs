use crate::error::{CliError, Result};
use crate::output::OutputFormat;
use finance_query::indicators::Indicator;
use finance_query::{Interval, TimeRange};
use ratatui::style::Color;

/// Categories of available indicators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndicatorCategory {
    MovingAverages,
    Oscillators,
    Trend,
    Volatility,
    Volume,
}

impl IndicatorCategory {
    pub fn all() -> Vec<Self> {
        vec![
            Self::MovingAverages,
            Self::Oscillators,
            Self::Trend,
            Self::Volatility,
            Self::Volume,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::MovingAverages => "Moving Averages",
            Self::Oscillators => "Oscillators",
            Self::Trend => "Trend",
            Self::Volatility => "Volatility",
            Self::Volume => "Volume",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::MovingAverages => Color::Cyan,
            Self::Oscillators => Color::Magenta,
            Self::Trend => Color::Yellow,
            Self::Volatility => Color::Red,
            Self::Volume => Color::Green,
        }
    }
}

/// Parameter definition for an indicator
#[derive(Debug, Clone)]
pub struct ParamDef {
    pub name: &'static str,
    pub description: &'static str,
    pub default: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,
}

/// Definition of an indicator with its parameters
#[derive(Debug, Clone)]
pub struct IndicatorDef {
    pub name: &'static str,
    pub code: &'static str,
    pub description: &'static str,
    pub category: IndicatorCategory,
    pub params: Vec<ParamDef>,
}

impl IndicatorDef {
    pub fn all() -> Vec<Self> {
        vec![
            // Moving Averages
            Self {
                name: "Simple Moving Average",
                code: "sma",
                description: "Average of closing prices over N periods",
                category: IndicatorCategory::MovingAverages,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Number of periods",
                    default: 20.0,
                    min: 2.0,
                    max: 500.0,
                    step: 1.0,
                }],
            },
            Self {
                name: "Exponential Moving Average",
                code: "ema",
                description: "Weighted average giving more weight to recent prices",
                category: IndicatorCategory::MovingAverages,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Number of periods",
                    default: 12.0,
                    min: 2.0,
                    max: 500.0,
                    step: 1.0,
                }],
            },
            Self {
                name: "Weighted Moving Average",
                code: "wma",
                description: "Linearly weighted moving average",
                category: IndicatorCategory::MovingAverages,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Number of periods",
                    default: 20.0,
                    min: 2.0,
                    max: 500.0,
                    step: 1.0,
                }],
            },
            Self {
                name: "Double EMA",
                code: "dema",
                description: "Double exponential moving average - reduces lag",
                category: IndicatorCategory::MovingAverages,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Number of periods",
                    default: 20.0,
                    min: 2.0,
                    max: 500.0,
                    step: 1.0,
                }],
            },
            Self {
                name: "Triple EMA",
                code: "tema",
                description: "Triple exponential moving average - even less lag",
                category: IndicatorCategory::MovingAverages,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Number of periods",
                    default: 20.0,
                    min: 2.0,
                    max: 500.0,
                    step: 1.0,
                }],
            },
            Self {
                name: "Hull Moving Average",
                code: "hma",
                description: "Fast and smooth moving average",
                category: IndicatorCategory::MovingAverages,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Number of periods",
                    default: 20.0,
                    min: 2.0,
                    max: 500.0,
                    step: 1.0,
                }],
            },
            Self {
                name: "Volume Weighted MA",
                code: "vwma",
                description: "Moving average weighted by volume",
                category: IndicatorCategory::MovingAverages,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Number of periods",
                    default: 20.0,
                    min: 2.0,
                    max: 500.0,
                    step: 1.0,
                }],
            },
            Self {
                name: "ALMA",
                code: "alma",
                description: "Arnaud Legoux Moving Average",
                category: IndicatorCategory::MovingAverages,
                params: vec![
                    ParamDef {
                        name: "Period",
                        description: "Number of periods",
                        default: 9.0,
                        min: 2.0,
                        max: 100.0,
                        step: 1.0,
                    },
                    ParamDef {
                        name: "Offset",
                        description: "Offset factor (0-1)",
                        default: 0.85,
                        min: 0.0,
                        max: 1.0,
                        step: 0.05,
                    },
                    ParamDef {
                        name: "Sigma",
                        description: "Sigma (smoothing)",
                        default: 6.0,
                        min: 1.0,
                        max: 20.0,
                        step: 0.5,
                    },
                ],
            },
            Self {
                name: "McGinley Dynamic",
                code: "mcginley",
                description: "Self-adjusting moving average",
                category: IndicatorCategory::MovingAverages,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Number of periods",
                    default: 14.0,
                    min: 2.0,
                    max: 500.0,
                    step: 1.0,
                }],
            },
            // Oscillators
            Self {
                name: "Relative Strength Index",
                code: "rsi",
                description: "Momentum oscillator measuring speed of price changes (0-100)",
                category: IndicatorCategory::Oscillators,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Number of periods",
                    default: 14.0,
                    min: 2.0,
                    max: 100.0,
                    step: 1.0,
                }],
            },
            Self {
                name: "Stochastic Oscillator",
                code: "stochastic",
                description: "Compares closing price to price range",
                category: IndicatorCategory::Oscillators,
                params: vec![
                    ParamDef {
                        name: "K Period",
                        description: "%K period",
                        default: 14.0,
                        min: 1.0,
                        max: 100.0,
                        step: 1.0,
                    },
                    ParamDef {
                        name: "K Slow",
                        description: "%K smoothing",
                        default: 3.0,
                        min: 1.0,
                        max: 20.0,
                        step: 1.0,
                    },
                    ParamDef {
                        name: "D Period",
                        description: "%D period",
                        default: 3.0,
                        min: 1.0,
                        max: 20.0,
                        step: 1.0,
                    },
                ],
            },
            Self {
                name: "Stochastic RSI",
                code: "stochrsi",
                description: "RSI with stochastic calculation",
                category: IndicatorCategory::Oscillators,
                params: vec![
                    ParamDef {
                        name: "RSI Period",
                        description: "RSI calculation period",
                        default: 14.0,
                        min: 2.0,
                        max: 100.0,
                        step: 1.0,
                    },
                    ParamDef {
                        name: "Stoch Period",
                        description: "Stochastic period",
                        default: 14.0,
                        min: 1.0,
                        max: 100.0,
                        step: 1.0,
                    },
                    ParamDef {
                        name: "K Period",
                        description: "%K smoothing",
                        default: 3.0,
                        min: 1.0,
                        max: 20.0,
                        step: 1.0,
                    },
                    ParamDef {
                        name: "D Period",
                        description: "%D period",
                        default: 3.0,
                        min: 1.0,
                        max: 20.0,
                        step: 1.0,
                    },
                ],
            },
            Self {
                name: "CCI",
                code: "cci",
                description: "Commodity Channel Index - measures deviation from mean",
                category: IndicatorCategory::Oscillators,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Number of periods",
                    default: 20.0,
                    min: 2.0,
                    max: 100.0,
                    step: 1.0,
                }],
            },
            Self {
                name: "Williams %R",
                code: "williams_r",
                description: "Momentum indicator similar to Stochastic (-100 to 0)",
                category: IndicatorCategory::Oscillators,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Number of periods",
                    default: 14.0,
                    min: 2.0,
                    max: 100.0,
                    step: 1.0,
                }],
            },
            Self {
                name: "Rate of Change",
                code: "roc",
                description: "Percentage change over N periods",
                category: IndicatorCategory::Oscillators,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Number of periods",
                    default: 12.0,
                    min: 1.0,
                    max: 100.0,
                    step: 1.0,
                }],
            },
            Self {
                name: "Momentum",
                code: "momentum",
                description: "Price difference over N periods",
                category: IndicatorCategory::Oscillators,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Number of periods",
                    default: 10.0,
                    min: 1.0,
                    max: 100.0,
                    step: 1.0,
                }],
            },
            Self {
                name: "Chande Momentum",
                code: "cmo",
                description: "Chande Momentum Oscillator",
                category: IndicatorCategory::Oscillators,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Number of periods",
                    default: 14.0,
                    min: 2.0,
                    max: 100.0,
                    step: 1.0,
                }],
            },
            Self {
                name: "Awesome Oscillator",
                code: "ao",
                description: "Measures market momentum",
                category: IndicatorCategory::Oscillators,
                params: vec![
                    ParamDef {
                        name: "Fast",
                        description: "Fast period",
                        default: 5.0,
                        min: 2.0,
                        max: 50.0,
                        step: 1.0,
                    },
                    ParamDef {
                        name: "Slow",
                        description: "Slow period",
                        default: 34.0,
                        min: 10.0,
                        max: 100.0,
                        step: 1.0,
                    },
                ],
            },
            Self {
                name: "MFI",
                code: "mfi",
                description: "Money Flow Index - volume-weighted RSI",
                category: IndicatorCategory::Oscillators,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Number of periods",
                    default: 14.0,
                    min: 2.0,
                    max: 100.0,
                    step: 1.0,
                }],
            },
            // Trend
            Self {
                name: "MACD",
                code: "macd",
                description: "Moving Average Convergence Divergence",
                category: IndicatorCategory::Trend,
                params: vec![
                    ParamDef {
                        name: "Fast",
                        description: "Fast EMA period",
                        default: 12.0,
                        min: 2.0,
                        max: 50.0,
                        step: 1.0,
                    },
                    ParamDef {
                        name: "Slow",
                        description: "Slow EMA period",
                        default: 26.0,
                        min: 10.0,
                        max: 100.0,
                        step: 1.0,
                    },
                    ParamDef {
                        name: "Signal",
                        description: "Signal line period",
                        default: 9.0,
                        min: 2.0,
                        max: 50.0,
                        step: 1.0,
                    },
                ],
            },
            Self {
                name: "ADX",
                code: "adx",
                description: "Average Directional Index - trend strength",
                category: IndicatorCategory::Trend,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Number of periods",
                    default: 14.0,
                    min: 2.0,
                    max: 100.0,
                    step: 1.0,
                }],
            },
            Self {
                name: "Aroon",
                code: "aroon",
                description: "Identifies trend changes and strength",
                category: IndicatorCategory::Trend,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Number of periods",
                    default: 25.0,
                    min: 2.0,
                    max: 100.0,
                    step: 1.0,
                }],
            },
            Self {
                name: "SuperTrend",
                code: "supertrend",
                description: "Trend-following indicator based on ATR",
                category: IndicatorCategory::Trend,
                params: vec![
                    ParamDef {
                        name: "Period",
                        description: "ATR period",
                        default: 10.0,
                        min: 1.0,
                        max: 100.0,
                        step: 1.0,
                    },
                    ParamDef {
                        name: "Multiplier",
                        description: "ATR multiplier",
                        default: 3.0,
                        min: 0.5,
                        max: 10.0,
                        step: 0.1,
                    },
                ],
            },
            Self {
                name: "Parabolic SAR",
                code: "psar",
                description: "Stop and reverse indicator",
                category: IndicatorCategory::Trend,
                params: vec![
                    ParamDef {
                        name: "Step",
                        description: "Acceleration factor step",
                        default: 0.02,
                        min: 0.01,
                        max: 0.1,
                        step: 0.01,
                    },
                    ParamDef {
                        name: "Max",
                        description: "Maximum acceleration",
                        default: 0.2,
                        min: 0.1,
                        max: 0.5,
                        step: 0.05,
                    },
                ],
            },
            Self {
                name: "Ichimoku Cloud",
                code: "ichimoku",
                description: "Comprehensive trend indicator",
                category: IndicatorCategory::Trend,
                params: vec![
                    ParamDef {
                        name: "Conversion",
                        description: "Conversion line period",
                        default: 9.0,
                        min: 1.0,
                        max: 50.0,
                        step: 1.0,
                    },
                    ParamDef {
                        name: "Base",
                        description: "Base line period",
                        default: 26.0,
                        min: 1.0,
                        max: 100.0,
                        step: 1.0,
                    },
                    ParamDef {
                        name: "Lagging",
                        description: "Lagging span period",
                        default: 52.0,
                        min: 1.0,
                        max: 200.0,
                        step: 1.0,
                    },
                    ParamDef {
                        name: "Displacement",
                        description: "Cloud displacement",
                        default: 26.0,
                        min: 1.0,
                        max: 100.0,
                        step: 1.0,
                    },
                ],
            },
            Self {
                name: "Coppock Curve",
                code: "coppock",
                description: "Long-term momentum indicator",
                category: IndicatorCategory::Trend,
                params: vec![
                    ParamDef {
                        name: "WMA Period",
                        description: "Weighted MA period",
                        default: 10.0,
                        min: 1.0,
                        max: 50.0,
                        step: 1.0,
                    },
                    ParamDef {
                        name: "Long ROC",
                        description: "Long ROC period",
                        default: 14.0,
                        min: 1.0,
                        max: 50.0,
                        step: 1.0,
                    },
                    ParamDef {
                        name: "Short ROC",
                        description: "Short ROC period",
                        default: 11.0,
                        min: 1.0,
                        max: 50.0,
                        step: 1.0,
                    },
                ],
            },
            // Volatility
            Self {
                name: "Bollinger Bands",
                code: "bollinger",
                description: "Volatility bands around a moving average",
                category: IndicatorCategory::Volatility,
                params: vec![
                    ParamDef {
                        name: "Period",
                        description: "Moving average period",
                        default: 20.0,
                        min: 2.0,
                        max: 100.0,
                        step: 1.0,
                    },
                    ParamDef {
                        name: "Std Dev",
                        description: "Standard deviation multiplier",
                        default: 2.0,
                        min: 0.5,
                        max: 5.0,
                        step: 0.1,
                    },
                ],
            },
            Self {
                name: "ATR",
                code: "atr",
                description: "Average True Range - volatility measure",
                category: IndicatorCategory::Volatility,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Number of periods",
                    default: 14.0,
                    min: 1.0,
                    max: 100.0,
                    step: 1.0,
                }],
            },
            Self {
                name: "True Range",
                code: "tr",
                description: "Single period true range",
                category: IndicatorCategory::Volatility,
                params: vec![],
            },
            Self {
                name: "Keltner Channels",
                code: "keltner",
                description: "Volatility-based envelope",
                category: IndicatorCategory::Volatility,
                params: vec![
                    ParamDef {
                        name: "Period",
                        description: "EMA period",
                        default: 20.0,
                        min: 2.0,
                        max: 100.0,
                        step: 1.0,
                    },
                    ParamDef {
                        name: "Multiplier",
                        description: "ATR multiplier",
                        default: 2.0,
                        min: 0.5,
                        max: 5.0,
                        step: 0.1,
                    },
                    ParamDef {
                        name: "ATR Period",
                        description: "ATR calculation period",
                        default: 10.0,
                        min: 1.0,
                        max: 50.0,
                        step: 1.0,
                    },
                ],
            },
            Self {
                name: "Donchian Channels",
                code: "donchian",
                description: "Highest high and lowest low over N periods",
                category: IndicatorCategory::Volatility,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Number of periods",
                    default: 20.0,
                    min: 2.0,
                    max: 100.0,
                    step: 1.0,
                }],
            },
            Self {
                name: "Choppiness Index",
                code: "chop",
                description: "Measures if market is trending or choppy",
                category: IndicatorCategory::Volatility,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Number of periods",
                    default: 14.0,
                    min: 2.0,
                    max: 100.0,
                    step: 1.0,
                }],
            },
            // Volume
            Self {
                name: "OBV",
                code: "obv",
                description: "On-Balance Volume - cumulative volume flow",
                category: IndicatorCategory::Volume,
                params: vec![],
            },
            Self {
                name: "VWAP",
                code: "vwap",
                description: "Volume Weighted Average Price",
                category: IndicatorCategory::Volume,
                params: vec![],
            },
            Self {
                name: "CMF",
                code: "cmf",
                description: "Chaikin Money Flow",
                category: IndicatorCategory::Volume,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Number of periods",
                    default: 20.0,
                    min: 2.0,
                    max: 100.0,
                    step: 1.0,
                }],
            },
            Self {
                name: "Chaikin Oscillator",
                code: "chaikin",
                description: "Accumulation/Distribution oscillator",
                category: IndicatorCategory::Volume,
                params: vec![],
            },
            Self {
                name: "A/D Line",
                code: "ad",
                description: "Accumulation/Distribution Line",
                category: IndicatorCategory::Volume,
                params: vec![],
            },
            Self {
                name: "Balance of Power",
                code: "bop",
                description: "Measures buying vs selling pressure",
                category: IndicatorCategory::Volume,
                params: vec![ParamDef {
                    name: "Period",
                    description: "Smoothing period (0 for raw)",
                    default: 0.0,
                    min: 0.0,
                    max: 50.0,
                    step: 1.0,
                }],
            },
        ]
    }
}

/// TUI screen states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Main screen - select indicator category and indicator
    IndicatorSelect,
    /// Configure indicator parameters
    ParamConfig,
    /// Configure chart settings (symbol, interval, range)
    ChartConfig,
    /// Confirmation before running
    Confirmation,
}

/// Which field is being edited in chart config
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartField {
    Symbol,
    Interval,
    Range,
}

impl ChartField {
    pub fn all() -> Vec<Self> {
        vec![Self::Symbol, Self::Interval, Self::Range]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Symbol => "Symbol",
            Self::Interval => "Interval",
            Self::Range => "Time Range",
        }
    }

    pub fn help(&self) -> &'static str {
        match self {
            Self::Symbol => "Stock ticker symbol (e.g., AAPL, TSLA, MSFT)",
            Self::Interval => "Candle interval: 1m, 5m, 15m, 1h, 1d, 1wk, 1mo",
            Self::Range => "Historical range: 1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, max",
        }
    }
}

/// Main application state
pub struct App {
    // Navigation
    pub screen: Screen,
    pub prev_screens: Vec<Screen>,

    // Chart configuration
    pub symbol: String,
    pub interval: Interval,
    pub range: TimeRange,
    pub format: OutputFormat,
    pub latest: bool,

    // Indicator selection
    pub category_idx: usize,
    pub indicator_idx: usize,
    pub selected_indicator: Option<IndicatorDef>,

    // Parameter editing
    pub param_idx: usize,
    pub param_values: Vec<f64>,

    // Chart config editing
    pub chart_field_idx: usize,
    pub editing: bool,
    pub edit_buffer: String,
    pub edit_error: Option<String>,

    // Available indicators
    pub indicators: Vec<IndicatorDef>,

    // Control
    pub should_quit: bool,
    pub confirmed: bool,
}

impl App {
    pub fn new(initial_symbol: Option<String>) -> Self {
        Self {
            screen: Screen::IndicatorSelect,
            prev_screens: Vec::new(),
            symbol: initial_symbol.unwrap_or_default().to_uppercase(),
            interval: Interval::OneDay,
            range: TimeRange::ThreeMonths,
            format: OutputFormat::Table,
            latest: false,
            category_idx: 0,
            indicator_idx: 0,
            selected_indicator: None,
            param_idx: 0,
            param_values: Vec::new(),
            chart_field_idx: 0,
            editing: false,
            edit_buffer: String::new(),
            edit_error: None,
            indicators: IndicatorDef::all(),
            should_quit: false,
            confirmed: false,
        }
    }

    /// Get indicators in current category
    pub fn indicators_in_category(&self) -> Vec<&IndicatorDef> {
        let categories = IndicatorCategory::all();
        let current_category = categories.get(self.category_idx).copied();

        self.indicators
            .iter()
            .filter(|ind| Some(ind.category) == current_category)
            .collect()
    }

    /// Count indicators in a category
    pub fn indicator_count_by_category(&self, cat: IndicatorCategory) -> usize {
        self.indicators
            .iter()
            .filter(|ind| ind.category == cat)
            .count()
    }

    /// Get currently selected indicator def
    pub fn current_indicator(&self) -> Option<&IndicatorDef> {
        let indicators = self.indicators_in_category();
        indicators.get(self.indicator_idx).copied()
    }

    /// Push a new screen
    pub fn push_screen(&mut self, screen: Screen) {
        self.prev_screens.push(self.screen);
        self.screen = screen;
    }

    /// Pop back to previous screen
    pub fn pop_screen(&mut self) {
        if let Some(prev) = self.prev_screens.pop() {
            self.screen = prev;
        }
    }

    /// Select the current indicator and move to param config
    pub fn select_indicator(&mut self) {
        if let Some(ind) = self.current_indicator().cloned() {
            self.param_values = ind.params.iter().map(|p| p.default).collect();
            self.param_idx = 0;

            let has_params = !ind.params.is_empty();
            self.selected_indicator = Some(ind);

            if has_params {
                self.push_screen(Screen::ParamConfig);
            } else {
                // No parameters, go straight to chart config
                self.push_screen(Screen::ChartConfig);
            }
        }
    }

    /// Start editing a chart field
    pub fn start_editing(&mut self) {
        let fields = ChartField::all();
        if let Some(field) = fields.get(self.chart_field_idx) {
            self.editing = true;
            self.edit_error = None;
            self.edit_buffer = match field {
                ChartField::Symbol => self.symbol.clone(),
                ChartField::Interval => interval_to_string(self.interval),
                ChartField::Range => range_to_string(self.range),
            };
        }
    }

    /// Finish editing and apply value
    pub fn finish_editing(&mut self) {
        let fields = ChartField::all();
        if let Some(field) = fields.get(self.chart_field_idx) {
            let result = match field {
                ChartField::Symbol => {
                    self.symbol = self.edit_buffer.trim().to_uppercase();
                    Ok(())
                }
                ChartField::Interval => match parse_interval(&self.edit_buffer) {
                    Ok(i) => {
                        self.interval = i;
                        Ok(())
                    }
                    Err(e) => Err(e.to_string()),
                },
                ChartField::Range => match parse_range(&self.edit_buffer) {
                    Ok(r) => {
                        self.range = r;
                        Ok(())
                    }
                    Err(e) => Err(e.to_string()),
                },
            };

            if let Err(e) = result {
                self.edit_error = Some(e);
            } else {
                self.editing = false;
                self.edit_buffer.clear();
            }
        }
    }

    /// Cancel editing
    pub fn cancel_editing(&mut self) {
        self.editing = false;
        self.edit_buffer.clear();
        self.edit_error = None;
    }

    /// Check if ready to run
    pub fn can_run(&self) -> bool {
        !self.symbol.is_empty() && self.selected_indicator.is_some()
    }

    /// Build the final indicator configuration
    pub fn build_config(&self) -> Result<super::IndicatorConfig> {
        let ind_def = self
            .selected_indicator
            .as_ref()
            .ok_or_else(|| CliError::InvalidArgument("No indicator selected".to_string()))?;

        let indicator = build_indicator(ind_def, &self.param_values)?;

        Ok(super::IndicatorConfig {
            symbol: self.symbol.clone(),
            indicator,
            interval: self.interval,
            range: self.range,
            format: self.format,
            latest: self.latest,
        })
    }

    /// Get current chart field value for display
    pub fn get_chart_field_value(&self, field: ChartField) -> String {
        match field {
            ChartField::Symbol => {
                if self.symbol.is_empty() {
                    "(not set)".to_string()
                } else {
                    self.symbol.clone()
                }
            }
            ChartField::Interval => interval_to_string(self.interval),
            ChartField::Range => range_to_string(self.range),
        }
    }
}

/// Build an Indicator from def and param values
fn build_indicator(def: &IndicatorDef, params: &[f64]) -> Result<Indicator> {
    match def.code {
        // Moving Averages
        "sma" => Ok(Indicator::Sma(
            params.first().copied().unwrap_or(20.0) as usize
        )),
        "ema" => Ok(Indicator::Ema(
            params.first().copied().unwrap_or(12.0) as usize
        )),
        "wma" => Ok(Indicator::Wma(
            params.first().copied().unwrap_or(20.0) as usize
        )),
        "dema" => Ok(Indicator::Dema(
            params.first().copied().unwrap_or(20.0) as usize
        )),
        "tema" => Ok(Indicator::Tema(
            params.first().copied().unwrap_or(20.0) as usize
        )),
        "hma" => Ok(Indicator::Hma(
            params.first().copied().unwrap_or(20.0) as usize
        )),
        "vwma" => Ok(Indicator::Vwma(
            params.first().copied().unwrap_or(20.0) as usize
        )),
        "alma" => Ok(Indicator::Alma {
            period: params.first().copied().unwrap_or(9.0) as usize,
            offset: params.get(1).copied().unwrap_or(0.85),
            sigma: params.get(2).copied().unwrap_or(6.0),
        }),
        "mcginley" => Ok(Indicator::McginleyDynamic(
            params.first().copied().unwrap_or(14.0) as usize,
        )),

        // Oscillators
        "rsi" => Ok(Indicator::Rsi(
            params.first().copied().unwrap_or(14.0) as usize
        )),
        "stochastic" => Ok(Indicator::Stochastic {
            k_period: params.first().copied().unwrap_or(14.0) as usize,
            k_slow: params.get(1).copied().unwrap_or(3.0) as usize,
            d_period: params.get(2).copied().unwrap_or(3.0) as usize,
        }),
        "stochrsi" => Ok(Indicator::StochasticRsi {
            rsi_period: params.first().copied().unwrap_or(14.0) as usize,
            stoch_period: params.get(1).copied().unwrap_or(14.0) as usize,
            k_period: params.get(2).copied().unwrap_or(3.0) as usize,
            d_period: params.get(3).copied().unwrap_or(3.0) as usize,
        }),
        "cci" => Ok(Indicator::Cci(
            params.first().copied().unwrap_or(20.0) as usize
        )),
        "williams_r" => Ok(Indicator::WilliamsR(
            params.first().copied().unwrap_or(14.0) as usize,
        )),
        "roc" => Ok(Indicator::Roc(
            params.first().copied().unwrap_or(12.0) as usize
        )),
        "momentum" => Ok(Indicator::Momentum(
            params.first().copied().unwrap_or(10.0) as usize
        )),
        "cmo" => Ok(Indicator::Cmo(
            params.first().copied().unwrap_or(14.0) as usize
        )),
        "ao" => Ok(Indicator::AwesomeOscillator {
            fast: params.first().copied().unwrap_or(5.0) as usize,
            slow: params.get(1).copied().unwrap_or(34.0) as usize,
        }),
        "mfi" => Ok(Indicator::Mfi(
            params.first().copied().unwrap_or(14.0) as usize
        )),

        // Trend
        "macd" => Ok(Indicator::Macd {
            fast: params.first().copied().unwrap_or(12.0) as usize,
            slow: params.get(1).copied().unwrap_or(26.0) as usize,
            signal: params.get(2).copied().unwrap_or(9.0) as usize,
        }),
        "adx" => Ok(Indicator::Adx(
            params.first().copied().unwrap_or(14.0) as usize
        )),
        "aroon" => Ok(Indicator::Aroon(
            params.first().copied().unwrap_or(25.0) as usize
        )),
        "supertrend" => Ok(Indicator::Supertrend {
            period: params.first().copied().unwrap_or(10.0) as usize,
            multiplier: params.get(1).copied().unwrap_or(3.0),
        }),
        "psar" => Ok(Indicator::ParabolicSar {
            step: params.first().copied().unwrap_or(0.02),
            max: params.get(1).copied().unwrap_or(0.2),
        }),
        "ichimoku" => Ok(Indicator::Ichimoku {
            conversion: params.first().copied().unwrap_or(9.0) as usize,
            base: params.get(1).copied().unwrap_or(26.0) as usize,
            lagging: params.get(2).copied().unwrap_or(52.0) as usize,
            displacement: params.get(3).copied().unwrap_or(26.0) as usize,
        }),
        "coppock" => Ok(Indicator::CoppockCurve {
            wma_period: params.first().copied().unwrap_or(10.0) as usize,
            long_roc: params.get(1).copied().unwrap_or(14.0) as usize,
            short_roc: params.get(2).copied().unwrap_or(11.0) as usize,
        }),

        // Volatility
        "bollinger" => Ok(Indicator::Bollinger {
            period: params.first().copied().unwrap_or(20.0) as usize,
            std_dev: params.get(1).copied().unwrap_or(2.0),
        }),
        "atr" => Ok(Indicator::Atr(
            params.first().copied().unwrap_or(14.0) as usize
        )),
        "tr" => Ok(Indicator::TrueRange),
        "keltner" => Ok(Indicator::KeltnerChannels {
            period: params.first().copied().unwrap_or(20.0) as usize,
            multiplier: params.get(1).copied().unwrap_or(2.0),
            atr_period: params.get(2).copied().unwrap_or(10.0) as usize,
        }),
        "donchian" => Ok(Indicator::DonchianChannels(
            params.first().copied().unwrap_or(20.0) as usize,
        )),
        "chop" => Ok(Indicator::ChoppinessIndex(
            params.first().copied().unwrap_or(14.0) as usize,
        )),

        // Volume
        "obv" => Ok(Indicator::Obv),
        "vwap" => Ok(Indicator::Vwap),
        "cmf" => Ok(Indicator::Cmf(
            params.first().copied().unwrap_or(20.0) as usize
        )),
        "chaikin" => Ok(Indicator::ChaikinOscillator),
        "ad" => Ok(Indicator::AccumulationDistribution),
        "bop" => {
            let period = params.first().copied().unwrap_or(0.0) as usize;
            Ok(Indicator::BalanceOfPower(if period == 0 {
                None
            } else {
                Some(period)
            }))
        }

        _ => Err(CliError::InvalidArgument(format!(
            "Unknown indicator code: {}",
            def.code
        ))),
    }
}

pub fn interval_to_string(interval: Interval) -> String {
    match interval {
        Interval::OneMinute => "1m".to_string(),
        Interval::FiveMinutes => "5m".to_string(),
        Interval::FifteenMinutes => "15m".to_string(),
        Interval::ThirtyMinutes => "30m".to_string(),
        Interval::OneHour => "1h".to_string(),
        Interval::OneDay => "1d".to_string(),
        Interval::OneWeek => "1wk".to_string(),
        Interval::OneMonth => "1mo".to_string(),
        Interval::ThreeMonths => "3mo".to_string(),
    }
}

pub fn range_to_string(range: TimeRange) -> String {
    match range {
        TimeRange::OneDay => "1d".to_string(),
        TimeRange::FiveDays => "5d".to_string(),
        TimeRange::OneMonth => "1mo".to_string(),
        TimeRange::ThreeMonths => "3mo".to_string(),
        TimeRange::SixMonths => "6mo".to_string(),
        TimeRange::OneYear => "1y".to_string(),
        TimeRange::TwoYears => "2y".to_string(),
        TimeRange::FiveYears => "5y".to_string(),
        TimeRange::TenYears => "10y".to_string(),
        TimeRange::YearToDate => "ytd".to_string(),
        TimeRange::Max => "max".to_string(),
    }
}

fn parse_interval(s: &str) -> Result<Interval> {
    match s.to_lowercase().trim() {
        "1m" => Ok(Interval::OneMinute),
        "5m" => Ok(Interval::FiveMinutes),
        "15m" => Ok(Interval::FifteenMinutes),
        "1h" => Ok(Interval::OneHour),
        "1d" => Ok(Interval::OneDay),
        "1wk" => Ok(Interval::OneWeek),
        "1mo" => Ok(Interval::OneMonth),
        _ => Err(CliError::InvalidArgument(format!(
            "Invalid interval '{}'. Valid: 1m, 5m, 15m, 1h, 1d, 1wk, 1mo",
            s
        ))),
    }
}

fn parse_range(s: &str) -> Result<TimeRange> {
    match s.to_lowercase().trim() {
        "1d" => Ok(TimeRange::OneDay),
        "5d" => Ok(TimeRange::FiveDays),
        "1mo" => Ok(TimeRange::OneMonth),
        "3mo" => Ok(TimeRange::ThreeMonths),
        "6mo" => Ok(TimeRange::SixMonths),
        "1y" => Ok(TimeRange::OneYear),
        "2y" => Ok(TimeRange::TwoYears),
        "5y" => Ok(TimeRange::FiveYears),
        "10y" => Ok(TimeRange::TenYears),
        "ytd" => Ok(TimeRange::YearToDate),
        "max" => Ok(TimeRange::Max),
        _ => Err(CliError::InvalidArgument(format!(
            "Invalid range '{}'. Valid: 1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max",
            s
        ))),
    }
}
