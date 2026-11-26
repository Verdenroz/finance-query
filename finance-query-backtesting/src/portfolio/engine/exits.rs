//! Forced exits: stop/take-profit/trailing checks and liquidation fills.

use crate::backtesting::config::BacktestConfig;
use crate::backtesting::result::SignalRecord;
use crate::backtesting::signal::{Signal, SignalDirection};
use crate::backtesting::strategy::Strategy;

use super::state::SymbolState;

pub(super) fn execute_forced_exit<S: Strategy>(
    config: &BacktestConfig,
    state: &mut SymbolState<S>,
    cash: &mut f64,
    timestamp: i64,
    fill_price: f64,
    exit_signal: Signal,
) -> bool {
    let Some(pos) = state.position.take() else {
        return false;
    };
    let exit_price_slipped = config.apply_exit_slippage(fill_price, pos.is_long());
    let exit_price = config.apply_exit_spread(exit_price_slipped, pos.is_long());
    let exit_comm = config.calculate_commission(pos.quantity, exit_price);
    let exit_tax = config.calculate_transaction_tax(exit_price * pos.quantity, !pos.is_long());
    let exit_reason = exit_signal.reason.clone();
    let exit_tags = exit_signal.tags.clone();
    let trade = pos.close_with_tax(timestamp, exit_price, exit_comm, exit_tax, exit_signal);
    if trade.is_long() {
        *cash += trade.exit_value() - exit_comm + trade.unreinvested_dividends;
    } else {
        *cash -= trade.exit_value() + exit_comm + exit_tax - trade.unreinvested_dividends;
    }
    state.realized_pnl += trade.pnl;
    state.trades.push(trade);
    state.hwm = None;
    state.extremes = None;
    state.signals.push(SignalRecord {
        timestamp,
        price: fill_price,
        direction: SignalDirection::Exit,
        strength: 1.0,
        reason: exit_reason,
        executed: true,
        tags: exit_tags,
    });
    true
}
