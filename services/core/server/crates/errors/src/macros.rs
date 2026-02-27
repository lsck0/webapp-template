/// Create a body user error.
#[macro_export]
macro_rules! body_error {
    ($err:expr) => {{
        use errors::*;
        ServerError::UserError(UserError::BodyError($err))
    }};
}

/// Create a user error.
#[macro_export]
macro_rules! user_error {
    ($err:expr) => {{
        use errors::{UserError::*, *};
        ServerError::UserError($err)
    }};
}

/// Early return with ok.
#[macro_export]
macro_rules! respond {
    ($val:expr) => {
        return Ok($val);
    };
}

/// Early return with an error.
#[macro_export]
macro_rules! bail {
    ($err:expr) => {
        return Err($err);
    };
}

/// Early return with an error, if the condition is not met.
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err);
        }
    };
}

/// Validate the length of a string field.
///
/// # Example
///
/// ```rust,no_run
/// let mut errors = vec![];
///
/// validate_string_length!(errors, self, title, 1, 100);
/// validate_string_length!(errors, self, content_field, 1, 255, "contentField");
/// ```
#[macro_export]
macro_rules! validate_string_length {
    ($errors:expr, $self:ident, $field:ident, $min:literal, $max:literal) => {
        let field = &$self.$field;
        if field.len() < $min || field.len() > $max {
            $errors.push(FieldError {
                field: stringify!($field).to_string(),
                reason: errors::FieldErrorReason::InvalidRange { min: $min, max: $max },
            });
        }
    };
    ($errors:expr, $self:ident, $field:ident, $min:literal, $max:literal, $field_name:literal) => {
        let field = &$self.$field;
        if field.len() < $min || field.len() > $max {
            $errors.push(FieldError {
                field: $field_name.to_string(),
                reason: errors::FieldErrorReason::InvalidRange { min: $min, max: $max },
            });
        }
    };
}

/// Validate the range of a number field.
///
/// # Example
///
/// ```rust,no_run
/// let mut errors = vec![];
///
/// validate_number_range!(errors, self, age, 1, 100);
/// validate_number_range!(errors, self, user_age, 1, 100, "UserAge");
/// ```
#[macro_export]
macro_rules! validate_number_range {
    ($errors:expr, $self:ident, $field:ident, $min:literal, $max:literal) => {
        let field = $self.$field;
        if (field as i64) < $min || (field as i64) > $max {
            $errors.push(FieldError {
                field: stringify!($field).to_string(),
                reason: errors::FieldErrorReason::InvalidRange { min: $min, max: $max },
            });
        }
    };
    ($errors:expr, $self:ident, $field:ident, $min:literal, $max:literal, $field_name:literal) => {
        let field = $self.$field;
        if (field as i64) < $min || (field as i64) > $max {
            $errors.push(FieldError {
                field: $field_name.to_string(),
                reason: errors::FieldErrorReason::InvalidRange { min: $min, max: $max },
            });
        }
    };
}
