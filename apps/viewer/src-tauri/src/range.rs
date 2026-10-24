use crate::error::CommandError;

pub fn validate(since_ms: i64, until_ms: i64) -> Result<(), CommandError> {
    if since_ms > until_ms {
        return Err(CommandError::invalid_argument(
            "開始時刻が終了時刻より後になっています",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate;

    #[test]
    fn rejects_reversed_range() {
        assert!(validate(10, 5).is_err());
        assert!(validate(5, 5).is_ok());
        assert!(validate(0, 10).is_ok());
    }
}
