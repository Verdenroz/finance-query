use super::helpers::{
    apply_cost_profile, detect_cost_profile, interval_to_string, parse_bool, range_to_string,
};
use super::{App, ConfigField};
use crate::backtest::types::bars_per_year_for_interval;
use crate::error::Result;
use crate::parse::{parse_interval, parse_range};

impl App {
    pub fn format_field_value(&self, field: ConfigField) -> String {
        match field {
            ConfigField::Symbol => {
                if self.config.symbol.is_empty() {
                    "(not set)".to_string()
                } else {
                    self.config.symbol.clone()
                }
            }
            ConfigField::Interval => interval_to_string(self.config.interval),
            ConfigField::Range => range_to_string(self.config.range),
            ConfigField::Capital => format!("${:.2}", self.config.capital),
            ConfigField::CostProfile => detect_cost_profile(&self.config).name().to_string(),
            ConfigField::Commission => format!("{:.2}%", self.config.commission * 100.0),
            ConfigField::CommissionFlat => format!("${:.2}", self.config.commission_flat),
            ConfigField::Slippage => format!("{:.2}%", self.config.slippage * 100.0),
            ConfigField::SpreadPct => format!("{:.2}%", self.config.spread_pct * 100.0),
            ConfigField::TransactionTaxPct => {
                format!("{:.2}%", self.config.transaction_tax_pct * 100.0)
            }
            ConfigField::AllowShort => format!("{}", self.config.allow_short),
            ConfigField::StopLoss => self
                .config
                .stop_loss
                .map(|v| format!("{:.1}%", v * 100.0))
                .unwrap_or_else(|| "None".to_string()),
            ConfigField::TakeProfit => self
                .config
                .take_profit
                .map(|v| format!("{:.1}%", v * 100.0))
                .unwrap_or_else(|| "None".to_string()),
            ConfigField::TrailingStop => self
                .config
                .trailing_stop
                .map(|v| format!("{:.1}%", v * 100.0))
                .unwrap_or_else(|| "None".to_string()),
            ConfigField::PositionSize => format!("{:.0}%", self.config.position_size * 100.0),
            ConfigField::MaxPositions => {
                if self.config.max_positions == 0 {
                    "Unlimited".to_string()
                } else {
                    self.config.max_positions.to_string()
                }
            }
            ConfigField::WarmupBars => {
                if self.config.strategy.warmup_bars == 0 {
                    "None".to_string()
                } else {
                    self.config.strategy.warmup_bars.to_string()
                }
            }
            ConfigField::RiskFreeRate => format!("{:.1}%", self.config.risk_free_rate * 100.0),
            ConfigField::MinSignalStrength => {
                format!("{:.1}%", self.config.min_signal_strength * 100.0)
            }
            ConfigField::CloseAtEnd => format!("{}", self.config.close_at_end),
            ConfigField::BarsPerYear => {
                if (self.config.bars_per_year - self.config.bars_per_year.round()).abs() < 1e-6 {
                    format!("{:.0}", self.config.bars_per_year)
                } else {
                    format!("{:.2}", self.config.bars_per_year)
                }
            }
            ConfigField::ReinvestDividends => format!("{}", self.config.reinvest_dividends),
            ConfigField::Benchmark => self
                .config
                .benchmark
                .clone()
                .unwrap_or_else(|| "None".to_string()),
            ConfigField::EntryOrderType => self.config.strategy.entry_order_type.name().to_string(),
            ConfigField::EntryPriceOffset => {
                format!(
                    "{:.2}%",
                    self.config.strategy.entry_price_offset_pct * 100.0
                )
            }
            ConfigField::EntryStopLimitGap => {
                format!(
                    "{:.2}%",
                    self.config.strategy.entry_stop_limit_gap_pct * 100.0
                )
            }
            ConfigField::EntryExpiryBars => match self.config.strategy.entry_expires_bars {
                None | Some(0) => "GTC".to_string(),
                Some(n) => format!("{} bars", n),
            },
            ConfigField::EntryBracketSL => self
                .config
                .strategy
                .entry_bracket_sl
                .map(|v| format!("{:.1}%", v * 100.0))
                .unwrap_or_else(|| "Global".to_string()),
            ConfigField::EntryBracketTP => self
                .config
                .strategy
                .entry_bracket_tp
                .map(|v| format!("{:.1}%", v * 100.0))
                .unwrap_or_else(|| "Global".to_string()),
            ConfigField::EntryBracketTrail => self
                .config
                .strategy
                .entry_bracket_trail
                .map(|v| format!("{:.1}%", v * 100.0))
                .unwrap_or_else(|| "Global".to_string()),
            ConfigField::ShortOrderType => self.config.strategy.short_order_type.name().to_string(),
            ConfigField::ShortPriceOffset => {
                format!(
                    "{:.2}%",
                    self.config.strategy.short_price_offset_pct * 100.0
                )
            }
            ConfigField::ShortExpiryBars => match self.config.strategy.short_expires_bars {
                None | Some(0) => "GTC".to_string(),
                Some(n) => format!("{} bars", n),
            },
            ConfigField::ShortBracketSL => self
                .config
                .strategy
                .short_bracket_sl
                .map(|v| format!("{:.1}%", v * 100.0))
                .unwrap_or_else(|| "Global".to_string()),
            ConfigField::ShortBracketTP => self
                .config
                .strategy
                .short_bracket_tp
                .map(|v| format!("{:.1}%", v * 100.0))
                .unwrap_or_else(|| "Global".to_string()),
            ConfigField::ShortBracketTrail => self
                .config
                .strategy
                .short_bracket_trail
                .map(|v| format!("{:.1}%", v * 100.0))
                .unwrap_or_else(|| "Global".to_string()),
            ConfigField::PortfolioSymbols => {
                if self.config.portfolio_symbols.is_empty() {
                    "(single-symbol mode)".to_string()
                } else {
                    self.config.portfolio_symbols.join(", ")
                }
            }
            ConfigField::RebalanceMode => self.config.rebalance_mode.name().to_string(),
        }
    }

    pub fn start_editing(&mut self) {
        self.editing = true;
        self.edit_error = None;
        let field = self.current_config_field();

        // Handle enum-cycle fields that don't use text input
        match field {
            ConfigField::CostProfile => {
                let current = detect_cost_profile(&self.config);
                apply_cost_profile(&mut self.config, current.next());
                self.editing = false;
                return;
            }
            ConfigField::EntryOrderType => {
                self.config.strategy.entry_order_type =
                    self.config.strategy.entry_order_type.cycle();
                self.editing = false;
                return;
            }
            ConfigField::ShortOrderType => {
                self.config.strategy.short_order_type =
                    self.config.strategy.short_order_type.cycle();
                self.editing = false;
                return;
            }
            ConfigField::RebalanceMode => {
                self.config.rebalance_mode = self.config.rebalance_mode.cycle();
                self.editing = false;
                return;
            }
            _ => {}
        }

        self.edit_buffer = match field {
            ConfigField::Symbol => self.config.symbol.clone(),
            ConfigField::Interval => interval_to_string(self.config.interval),
            ConfigField::Range => range_to_string(self.config.range),
            ConfigField::Capital => format!("{}", self.config.capital),
            ConfigField::CostProfile => String::new(),
            ConfigField::Commission => format!("{}", self.config.commission * 100.0),
            ConfigField::CommissionFlat => format!("{}", self.config.commission_flat),
            ConfigField::Slippage => format!("{}", self.config.slippage * 100.0),
            ConfigField::SpreadPct => format!("{}", self.config.spread_pct * 100.0),
            ConfigField::TransactionTaxPct => {
                format!("{}", self.config.transaction_tax_pct * 100.0)
            }
            ConfigField::AllowShort => format!("{}", self.config.allow_short),
            ConfigField::StopLoss => self
                .config
                .stop_loss
                .map(|v| format!("{}", v * 100.0))
                .unwrap_or_default(),
            ConfigField::TakeProfit => self
                .config
                .take_profit
                .map(|v| format!("{}", v * 100.0))
                .unwrap_or_default(),
            ConfigField::TrailingStop => self
                .config
                .trailing_stop
                .map(|v| format!("{}", v * 100.0))
                .unwrap_or_default(),
            ConfigField::PositionSize => format!("{}", self.config.position_size * 100.0),
            ConfigField::MaxPositions => format!("{}", self.config.max_positions),
            ConfigField::WarmupBars => format!("{}", self.config.strategy.warmup_bars),
            ConfigField::RiskFreeRate => format!("{}", self.config.risk_free_rate * 100.0),
            ConfigField::MinSignalStrength => {
                format!("{}", self.config.min_signal_strength * 100.0)
            }
            ConfigField::CloseAtEnd => format!("{}", self.config.close_at_end),
            ConfigField::BarsPerYear => format!("{}", self.config.bars_per_year),
            ConfigField::ReinvestDividends => format!("{}", self.config.reinvest_dividends),
            ConfigField::Benchmark => self.config.benchmark.clone().unwrap_or_default(),
            ConfigField::EntryOrderType
            | ConfigField::ShortOrderType
            | ConfigField::RebalanceMode => {
                // Already handled above by enum-cycle — unreachable
                String::new()
            }
            ConfigField::EntryPriceOffset => {
                format!("{}", self.config.strategy.entry_price_offset_pct * 100.0)
            }
            ConfigField::EntryStopLimitGap => {
                format!("{}", self.config.strategy.entry_stop_limit_gap_pct * 100.0)
            }
            ConfigField::EntryExpiryBars => self
                .config
                .strategy
                .entry_expires_bars
                .map(|n| n.to_string())
                .unwrap_or_default(),
            ConfigField::EntryBracketSL => self
                .config
                .strategy
                .entry_bracket_sl
                .map(|v| format!("{}", v * 100.0))
                .unwrap_or_default(),
            ConfigField::EntryBracketTP => self
                .config
                .strategy
                .entry_bracket_tp
                .map(|v| format!("{}", v * 100.0))
                .unwrap_or_default(),
            ConfigField::EntryBracketTrail => self
                .config
                .strategy
                .entry_bracket_trail
                .map(|v| format!("{}", v * 100.0))
                .unwrap_or_default(),
            ConfigField::ShortPriceOffset => {
                format!("{}", self.config.strategy.short_price_offset_pct * 100.0)
            }
            ConfigField::ShortExpiryBars => self
                .config
                .strategy
                .short_expires_bars
                .map(|n| n.to_string())
                .unwrap_or_default(),
            ConfigField::ShortBracketSL => self
                .config
                .strategy
                .short_bracket_sl
                .map(|v| format!("{}", v * 100.0))
                .unwrap_or_default(),
            ConfigField::ShortBracketTP => self
                .config
                .strategy
                .short_bracket_tp
                .map(|v| format!("{}", v * 100.0))
                .unwrap_or_default(),
            ConfigField::ShortBracketTrail => self
                .config
                .strategy
                .short_bracket_trail
                .map(|v| format!("{}", v * 100.0))
                .unwrap_or_default(),
            ConfigField::PortfolioSymbols => self.config.portfolio_symbols.join(", "),
        };
    }

    pub fn finish_editing(&mut self) {
        let value = self.edit_buffer.trim();
        let field = self.current_config_field();

        let result: Result<()> = (|| {
            match field {
                ConfigField::Symbol => {
                    self.config.symbol = value.to_uppercase();
                }
                ConfigField::Interval => {
                    let interval = parse_interval(value)?;
                    self.config.interval = interval;
                    // Keep annualisation assumptions aligned with the selected interval
                    // until the user explicitly overrides Bars / Year.
                    self.config.bars_per_year = bars_per_year_for_interval(interval);
                }
                ConfigField::Range => {
                    self.config.range = parse_range(value)?;
                }
                ConfigField::Capital => {
                    let cap: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if cap <= 0.0 {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Capital must be positive".into(),
                        ));
                    }
                    self.config.capital = cap;
                }
                ConfigField::CostProfile => {
                    // Handled in start_editing by cycling
                }
                ConfigField::Commission => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if v < 0.0 {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Commission cannot be negative".into(),
                        ));
                    }
                    self.config.commission = v / 100.0;
                }
                ConfigField::CommissionFlat => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if v < 0.0 {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Flat commission cannot be negative".into(),
                        ));
                    }
                    self.config.commission_flat = v;
                }
                ConfigField::Slippage => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if v < 0.0 {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Slippage cannot be negative".into(),
                        ));
                    }
                    self.config.slippage = v / 100.0;
                }
                ConfigField::SpreadPct => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if !(0.0..=100.0).contains(&v) {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Spread must be 0-100%".into(),
                        ));
                    }
                    self.config.spread_pct = v / 100.0;
                }
                ConfigField::TransactionTaxPct => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if !(0.0..=100.0).contains(&v) {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Transaction tax must be 0-100%".into(),
                        ));
                    }
                    self.config.transaction_tax_pct = v / 100.0;
                }
                ConfigField::MaxPositions => {
                    let v: usize = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument(
                            "Must be a whole number (0 = unlimited)".into(),
                        )
                    })?;
                    self.config.max_positions = v;
                }
                ConfigField::WarmupBars => {
                    let v: usize = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument(
                            "Must be a whole number (0 = no warmup)".into(),
                        )
                    })?;
                    self.config.strategy.warmup_bars = v;
                }
                ConfigField::AllowShort => {
                    self.config.allow_short = parse_bool(value)?;
                }
                ConfigField::StopLoss => {
                    if value.is_empty() {
                        self.config.stop_loss = None;
                    } else {
                        let v: f64 = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument("Invalid number".into())
                        })?;
                        if v <= 0.0 || v > 100.0 {
                            return Err(crate::error::CliError::InvalidArgument(
                                "Stop loss must be between 0 and 100%".into(),
                            ));
                        }
                        // Reject if stop-loss is at or below round-trip costs.
                        // A stop at or below break-even means every stopped-out trade
                        // is guaranteed to lose money on fees alone.
                        // Include all friction sources: commission, slippage, spread (half
                        // applied each side → full spread round-trip), and transaction tax
                        // (buy-only, so counted once).
                        let round_trip_pct = (self.config.commission * 2.0
                            + self.config.slippage * 2.0
                            + self.config.spread_pct
                            + self.config.transaction_tax_pct)
                            * 100.0;
                        if v <= round_trip_pct {
                            return Err(crate::error::CliError::InvalidArgument(format!(
                                "Stop loss {v:.2}% ≤ round-trip cost {round_trip_pct:.2}% \
                                 (2× commission + 2× slippage + spread + transaction tax). \
                                 A stopped-out trade loses {v:.2}% plus {round_trip_pct:.2}% \
                                 in fees — no trade can profit after costs at this stop level."
                            )));
                        }
                        self.config.stop_loss = Some(v / 100.0);
                    }
                }
                ConfigField::TakeProfit => {
                    if value.is_empty() {
                        self.config.take_profit = None;
                    } else {
                        let v: f64 = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument("Invalid number".into())
                        })?;
                        if v <= 0.0 || v > 1000.0 {
                            return Err(crate::error::CliError::InvalidArgument(
                                "Take profit must be between 0 and 1000%".into(),
                            ));
                        }
                        self.config.take_profit = Some(v / 100.0);
                    }
                }
                ConfigField::TrailingStop => {
                    if value.is_empty() {
                        self.config.trailing_stop = None;
                    } else {
                        let v: f64 = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument("Invalid number".into())
                        })?;
                        if v <= 0.0 || v > 100.0 {
                            return Err(crate::error::CliError::InvalidArgument(
                                "Trailing stop must be 0-100%".into(),
                            ));
                        }
                        self.config.trailing_stop = Some(v / 100.0);
                    }
                }
                ConfigField::PositionSize => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if v <= 0.0 || v > 100.0 {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Position size must be 0-100%".into(),
                        ));
                    }
                    self.config.position_size = v / 100.0;
                }
                ConfigField::RiskFreeRate => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if !(0.0..=100.0).contains(&v) {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Risk-free rate must be 0-100%".into(),
                        ));
                    }
                    self.config.risk_free_rate = v / 100.0;
                }
                ConfigField::MinSignalStrength => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if !(0.0..=100.0).contains(&v) {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Min signal strength must be 0-100%".into(),
                        ));
                    }
                    self.config.min_signal_strength = v / 100.0;
                }
                ConfigField::CloseAtEnd => {
                    self.config.close_at_end = parse_bool(value)?;
                }
                ConfigField::BarsPerYear => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if v <= 0.0 {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Bars per year must be positive".into(),
                        ));
                    }
                    self.config.bars_per_year = v;
                }
                ConfigField::ReinvestDividends => {
                    self.config.reinvest_dividends = parse_bool(value)?;
                }
                ConfigField::Benchmark => {
                    let sym = value.trim().to_uppercase();
                    self.config.benchmark = if sym.is_empty() { None } else { Some(sym) };
                }
                ConfigField::EntryOrderType => {
                    // Handled in start_editing by cycling
                }
                ConfigField::EntryPriceOffset => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if !(0.0..=100.0).contains(&v) {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Offset must be 0-100%".into(),
                        ));
                    }
                    self.config.strategy.entry_price_offset_pct = v / 100.0;
                }
                ConfigField::EntryStopLimitGap => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if !(0.0..=100.0).contains(&v) {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Gap must be 0-100%".into(),
                        ));
                    }
                    self.config.strategy.entry_stop_limit_gap_pct = v / 100.0;
                }
                ConfigField::EntryExpiryBars => {
                    if value.is_empty() {
                        self.config.strategy.entry_expires_bars = None;
                    } else {
                        let v: usize = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument(
                                "Entry expiry must be a whole number of bars (0 = GTC)".into(),
                            )
                        })?;
                        self.config.strategy.entry_expires_bars =
                            if v == 0 { None } else { Some(v) };
                    }
                }
                ConfigField::EntryBracketSL => {
                    if value.is_empty() {
                        self.config.strategy.entry_bracket_sl = None;
                    } else {
                        let v: f64 = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument("Invalid number".into())
                        })?;
                        if v <= 0.0 || v > 100.0 {
                            return Err(crate::error::CliError::InvalidArgument(
                                "Must be 0-100%".into(),
                            ));
                        }
                        self.config.strategy.entry_bracket_sl = Some(v / 100.0);
                    }
                }
                ConfigField::EntryBracketTP => {
                    if value.is_empty() {
                        self.config.strategy.entry_bracket_tp = None;
                    } else {
                        let v: f64 = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument("Invalid number".into())
                        })?;
                        if v <= 0.0 || v > 1000.0 {
                            return Err(crate::error::CliError::InvalidArgument(
                                "Must be 0-1000%".into(),
                            ));
                        }
                        self.config.strategy.entry_bracket_tp = Some(v / 100.0);
                    }
                }
                ConfigField::EntryBracketTrail => {
                    if value.is_empty() {
                        self.config.strategy.entry_bracket_trail = None;
                    } else {
                        let v: f64 = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument("Invalid number".into())
                        })?;
                        if v <= 0.0 || v > 100.0 {
                            return Err(crate::error::CliError::InvalidArgument(
                                "Must be 0-100%".into(),
                            ));
                        }
                        self.config.strategy.entry_bracket_trail = Some(v / 100.0);
                    }
                }
                ConfigField::ShortOrderType => {
                    // Handled in start_editing by cycling
                }
                ConfigField::ShortPriceOffset => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if !(0.0..=100.0).contains(&v) {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Offset must be 0-100%".into(),
                        ));
                    }
                    self.config.strategy.short_price_offset_pct = v / 100.0;
                }
                ConfigField::ShortExpiryBars => {
                    if value.is_empty() {
                        self.config.strategy.short_expires_bars = None;
                    } else {
                        let v: usize = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument(
                                "Short expiry must be a whole number of bars (0 = GTC)".into(),
                            )
                        })?;
                        self.config.strategy.short_expires_bars =
                            if v == 0 { None } else { Some(v) };
                    }
                }
                ConfigField::ShortBracketSL => {
                    if value.is_empty() {
                        self.config.strategy.short_bracket_sl = None;
                    } else {
                        let v: f64 = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument("Invalid number".into())
                        })?;
                        if v <= 0.0 || v > 100.0 {
                            return Err(crate::error::CliError::InvalidArgument(
                                "Must be 0-100%".into(),
                            ));
                        }
                        self.config.strategy.short_bracket_sl = Some(v / 100.0);
                    }
                }
                ConfigField::ShortBracketTP => {
                    if value.is_empty() {
                        self.config.strategy.short_bracket_tp = None;
                    } else {
                        let v: f64 = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument("Invalid number".into())
                        })?;
                        if v <= 0.0 || v > 1000.0 {
                            return Err(crate::error::CliError::InvalidArgument(
                                "Must be 0-1000%".into(),
                            ));
                        }
                        self.config.strategy.short_bracket_tp = Some(v / 100.0);
                    }
                }
                ConfigField::ShortBracketTrail => {
                    if value.is_empty() {
                        self.config.strategy.short_bracket_trail = None;
                    } else {
                        let v: f64 = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument("Invalid number".into())
                        })?;
                        if v <= 0.0 || v > 100.0 {
                            return Err(crate::error::CliError::InvalidArgument(
                                "Must be 0-100%".into(),
                            ));
                        }
                        self.config.strategy.short_bracket_trail = Some(v / 100.0);
                    }
                }
                ConfigField::PortfolioSymbols => {
                    // Parse comma-separated symbols, trim whitespace, uppercase each.
                    self.config.portfolio_symbols = value
                        .split(',')
                        .map(|s| s.trim().to_uppercase())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                ConfigField::RebalanceMode => {
                    // Handled in start_editing by cycling — unreachable via text editing.
                }
            }
            Ok(())
        })();

        match result {
            Ok(()) => {
                self.editing = false;
                self.edit_buffer.clear();
                self.edit_error = None;
            }
            Err(e) => {
                self.edit_error = Some(e.to_string());
            }
        }
    }

    pub fn cancel_editing(&mut self) {
        self.editing = false;
        self.edit_buffer.clear();
        self.edit_error = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use finance_query::Interval;

    fn field_index(field: ConfigField) -> usize {
        ConfigField::all()
            .iter()
            .position(|f| *f == field)
            .expect("field should exist")
    }

    #[test]
    fn interval_edit_updates_default_bars_per_year() {
        let mut app = App::new(None);
        app.config.bars_per_year = 999.0;
        app.config_field_idx = field_index(ConfigField::Interval);
        app.edit_buffer = "1wk".to_string();

        app.finish_editing();

        assert_eq!(app.config.interval, Interval::OneWeek);
        assert!(
            (app.config.bars_per_year - bars_per_year_for_interval(Interval::OneWeek)).abs() < 1e-9
        );
        assert!(app.edit_error.is_none());
    }

    #[test]
    fn min_signal_strength_parses_percent_to_fraction() {
        let mut app = App::new(None);
        app.config_field_idx = field_index(ConfigField::MinSignalStrength);
        app.edit_buffer = "25".to_string();

        app.finish_editing();

        assert!((app.config.min_signal_strength - 0.25).abs() < 1e-9);
        assert!(app.edit_error.is_none());
    }

    #[test]
    fn close_at_end_parses_boolean() {
        let mut app = App::new(None);
        app.config.close_at_end = true;
        app.config_field_idx = field_index(ConfigField::CloseAtEnd);
        app.edit_buffer = "false".to_string();

        app.finish_editing();

        assert!(!app.config.close_at_end);
        assert!(app.edit_error.is_none());
    }

    #[test]
    fn bars_per_year_must_be_positive() {
        let mut app = App::new(None);
        app.config.bars_per_year = 252.0;
        app.config_field_idx = field_index(ConfigField::BarsPerYear);
        app.edit_buffer = "0".to_string();

        app.finish_editing();

        assert!((app.config.bars_per_year - 252.0).abs() < 1e-9);
        let err = app.edit_error.expect("validation error expected");
        assert!(err.contains("positive"));
    }
}
