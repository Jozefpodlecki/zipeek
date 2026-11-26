#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScoringPolicy {
    pub required_days: u16,
    pub base_daily_successes: u16,
    pub debt_eligibility_days: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyError {
    InvalidRequiredDays,
    InvalidDailyQuota,
    InvalidEligibilityThreshold,
    DailyQuotaOverflow,
}

impl Default for ScoringPolicy {
    fn default() -> Self {
        Self {
            required_days: 7,
            base_daily_successes: 10,
            debt_eligibility_days: 3,
        }
    }
}

impl ScoringPolicy {
    pub const MAXIMUM_DEBT_UNITS: u16 = 5;
    pub const SUCCESSES_PER_DEBT_UNIT: u16 = 2;

    pub fn validate(self) -> Result<Self, PolicyError> {
        if self.required_days == 0 {
            return Err(PolicyError::InvalidRequiredDays);
        }
        if self.base_daily_successes == 0 {
            return Err(PolicyError::InvalidDailyQuota);
        }
        if !(2..=5).contains(&self.debt_eligibility_days)
            || self.debt_eligibility_days > self.required_days
        {
            return Err(PolicyError::InvalidEligibilityThreshold);
        }
        if self.base_daily_successes.checked_add(10).is_none() {
            return Err(PolicyError::DailyQuotaOverflow);
        }
        Ok(self)
    }

    pub fn daily_quota(self, debt: u16) -> u16 {
        self.base_daily_successes
            + Self::SUCCESSES_PER_DEBT_UNIT * debt.min(Self::MAXIMUM_DEBT_UNITS)
    }
}
