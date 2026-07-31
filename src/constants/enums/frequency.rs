/// Frequency for financial data (annual or quarterly)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Frequency {
    /// Annual financial data
    Annual,
    /// Quarterly financial data
    Quarterly,
}

impl Frequency {
    /// Convert frequency to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Frequency::Annual => "annual",
            Frequency::Quarterly => "quarterly",
        }
    }

    /// Build a fundamental type string with frequency prefix
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::Frequency;
    ///
    /// let field = Frequency::Annual.prefix("TotalRevenue");
    /// assert_eq!(field, "annualTotalRevenue");
    /// ```
    pub fn prefix(&self, field: &str) -> String {
        format!("{}{}", self.as_str(), field)
    }
}

impl std::str::FromStr for Frequency {
    type Err = ();

    /// Case-insensitive; accepts `as_str()`'s canonical form plus common
    /// shorthands (`"year"`/`"yearly"`, `"quarter"`/`"q"`).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "annual" | "yearly" | "year" => Ok(Frequency::Annual),
            "quarterly" | "quarter" | "q" => Ok(Frequency::Quarterly),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frequency_from_str_round_trips_as_str() {
        for frequency in [Frequency::Annual, Frequency::Quarterly] {
            assert_eq!(frequency.as_str().parse(), Ok(frequency));
        }
        assert_eq!("YEARLY".parse(), Ok(Frequency::Annual));
        assert_eq!("q".parse(), Ok(Frequency::Quarterly));
        assert_eq!("bogus".parse::<Frequency>(), Err(()));
    }
}
